// Solfachord Elegance
use crate::prelude::*;
use crate::testing::{Check, DECK, HAND, PZONE};

const ID: u32 = 27870337;

pub const LISTED_SERIES: &[u16] = &[SET_SOLFACHORD];
// DoSolfachord Cutia and FaSolfachord Fancia (Scales 8 and 5) in the Deck and the hand to place, and in the Pendulum Zones.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, HAND),
    (55226153, DECK),
    (55226153, DECK),
    (54100561, DECK),
    (54100561, DECK),
    (55226153, HAND),
    (55226153, PZONE),
    (54100561, PZONE),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_property(duel, EFFECT_FLAG_LIMIT_ZONE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_count_limit_full(duel, 1, ID, EFFECT_COUNT_CODE_OATH);
    e1.set_target(duel, target);
    e1.set_operation(duel, activate);
    e1.set_value(duel, Value::ActivateZone(zones));
    c.register_effect(duel, e1);
}

fn zones(duel: &Duel, a: &Args) -> i32 {
    let tp = a.tp;
    let mut zone = 0xff;
    let place_g = duel.get_matching_group(plfilter, tp, LOCATION_DECK, 0, None);
    let left_pend = duel.check_location(tp, LOCATION_PZONE, 0);
    let right_pend = duel.check_location(tp, LOCATION_PZONE, 1);
    let b1 = duel.check_pendulum_zones(tp) && !place_g.is_empty();
    let b2 = duel.is_existing_matching_card(plfilter, tp, LOCATION_HAND, 0, 1, None)
        && left_pend
        && right_pend
        && aux::select_unselect_group_check(duel, place_g, 2, 2, &rescon);
    if b1 && !b2 && !(left_pend && right_pend) {
        zone = 0xe;
    }
    zone
}

fn plfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_SOLFACHORD]) && c.is_type(duel, TYPE_PENDULUM) && !c.is_forbidden(duel)
}

fn rescon(duel: &Duel, sg: CardSet, _: CardSet, _: Option<CardId>) -> (bool, bool) {
    (
        sg.is_exists(duel, CardId::is_even_scale, 1, None) && sg.is_exists(duel, CardId::is_odd_scale, 1, None),
        false,
    )
}

fn toextra_group(duel: &Duel, tp: u8) -> CardSet {
    duel.get_matching_group(|c, duel| !c.is_forbidden(duel), tp, LOCATION_PZONE, 0, None)
}

fn target(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let place_g = duel.get_matching_group(plfilter, tp, LOCATION_DECK, 0, None);
    let toextra_g = toextra_group(duel, tp);
    let b1 = duel.check_pendulum_zones(tp) && !place_g.is_empty();
    let b2 = duel.is_existing_matching_card(plfilter, tp, LOCATION_HAND, 0, 1, None)
        && duel.check_location(tp, LOCATION_PZONE, 0)
        && duel.check_location(tp, LOCATION_PZONE, 1)
        && aux::select_unselect_group_check(duel, place_g, 2, 2, &rescon);
    let b3 = duel.is_player_can_draw(tp, 2) && aux::select_unselect_group_check(duel, toextra_g, 2, 2, &rescon);
    if chk == 0 {
        return Ok(b1 || b2 || b3);
    }
    let op = duel.select_effect(
        tp,
        &[(b1, aux::stringid(ID, 1)), (b2, aux::stringid(ID, 2)), (b3, aux::stringid(ID, 3))],
    )?;
    let op = op.map_or(0, |op| op as i64);
    e.set_label(duel, op);
    match op {
        1 => e.set_category(duel, 0),
        2 => {
            e.set_category(duel, CATEGORY_TOEXTRA);
            duel.set_operation_info(0, CATEGORY_TOEXTRA, None, 1, tp, LOCATION_HAND as i32);
        }
        3 => {
            e.set_category(duel, CATEGORY_TOEXTRA + CATEGORY_DRAW);
            duel.set_target_player(tp);
            duel.set_target_param(2);
            duel.set_operation_info(0, CATEGORY_TOEXTRA, None, 2, tp, LOCATION_PZONE as i32);
            duel.set_operation_info(0, CATEGORY_DRAW, None, 0, tp, 2);
        }
        _ => {}
    }
    Ok(false)
}

fn activate(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    match a.e.get_label(duel) {
        1 => {
            if !duel.check_pendulum_zones(tp) {
                return Ok(());
            }
            duel.hint(HINT_SELECTMSG, tp, HINTMSG_TOFIELD);
            let g = duel.select_matching_card(tp, plfilter, tp, LOCATION_DECK, 0, 1, 1, None)?;
            if let Some(sc) = g.first() {
                duel.move_to_field(sc, tp, tp, LOCATION_PZONE, POS_FACEUP, true)?;
            }
        }
        2 => {
            duel.hint(HINT_SELECTMSG, tp, aux::stringid(ID, 4));
            let g = duel.select_matching_card(tp, plfilter, tp, LOCATION_HAND, 0, 1, 1, None)?;
            if !(duel.send_to_extra_p(g, tp, REASON_EFFECT)? > 0
                && duel.check_location(tp, LOCATION_PZONE, 0)
                && duel.check_location(tp, LOCATION_PZONE, 1))
            {
                return Ok(());
            }
            let place_g = duel.get_matching_group(plfilter, tp, LOCATION_DECK, 0, None);
            if place_g.len() < 2 {
                return Ok(());
            }
            let sg = aux::select_unselect_group(duel, place_g, 2, 2, &rescon, tp, HINTMSG_TOFIELD)?;
            if sg.len() == 2 {
                duel.break_effect()?;
                for pc in sg {
                    duel.move_to_field(pc, tp, tp, LOCATION_PZONE, POS_FACEUP, true)?;
                }
            }
        }
        3 => {
            let toextra_g = toextra_group(duel, tp);
            if toextra_g.len() == 2
                && aux::select_unselect_group_check(duel, toextra_g, 2, 2, &rescon)
                && duel.send_to_extra_p(toextra_g, tp, REASON_EFFECT)? > 0
                && duel
                    .get_operated_group()
                    .filter_count(duel, |c, duel| c.is_location(duel, LOCATION_EXTRA), None)
                    == 2
            {
                stock::draw(duel, a)?;
            }
        }
        _ => {}
    }
    Ok(())
}
