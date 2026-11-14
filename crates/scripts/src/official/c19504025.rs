// Enneacraft Reset
use crate::prelude::*;
use crate::testing::{Check, DECK, GRAVE, HAND, MZONE, OPPONENT, PZONE};

const ID: u32 = 19504025;

pub const LISTED_NAMES: &[u32] = &[17621695];
pub const LISTED_SERIES: &[u16] = &[SET_ENNEACRAFT];

// In the hand and GY with Enneapolis in the Deck, Trito Enneacraft on the field and in a Pendulum Zone, and the opponent's Pot of Greed.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, GRAVE),
    (ID, GRAVE),
    (17621695, DECK),
    (17621695, DECK),
    (44716748, MZONE),
    (44716748, MZONE),
    (44716748, PZONE),
    (55144522, HAND | OPPONENT),
    (55144522, HAND | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_DISABLE);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_count_limit_full(duel, 1, ID, EFFECT_COUNT_CODE_OATH);
    e1.set_target(duel, target);
    e1.set_operation(duel, activate);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_POSITION + CATEGORY_SET);
    e2.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e2.set_property_full(duel, EFFECT_FLAG_DELAY, EFFECT_FLAG2_CHECK_SIMULTANEOUS);
    e2.set_code(duel, EVENT_CHAINING);
    e2.set_range(duel, LOCATION_GRAVE);
    e2.set_condition(duel, |_, a| a.ep == 1 - a.tp);
    e2.set_cost(duel, Cost::self_banish);
    e2.set_target(duel, postg);
    e2.set_operation(duel, posop);
    c.register_effect(duel, e2);
}

fn plfilter(c: CardId, duel: &Duel) -> bool {
    c.is_code(duel, 17621695) && !c.is_forbidden(duel)
}

fn disable_chk(duel: &Duel, tp: u8, pzone_card_count: usize) -> bool {
    duel.is_existing_matching_card(
        aux::faceup_filter(|c, duel| c.is_code(duel, 17621695)),
        tp,
        LOCATION_ONFIELD,
        0,
        1,
        None,
    ) && pzone_card_count > 0
        && duel.is_existing_matching_card(|c, duel| c.is_negatable(duel), tp, 0, LOCATION_ONFIELD, 1, None)
}

fn target(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(disable_chk(duel, tp, duel.get_field_group_count(tp, LOCATION_PZONE, 0))
            || duel.is_existing_matching_card(plfilter, tp, LOCATION_DECK | LOCATION_GRAVE, 0, 1, None));
    }
    duel.set_possible_operation_info(0, CATEGORY_DISABLE, None, 1, 1 - tp, LOCATION_ONFIELD as i32);
    Ok(false)
}

fn activate(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    let place_chk = duel.is_existing_matching_card(aux::necro_valley_filter(plfilter), tp, LOCATION_DECK | LOCATION_GRAVE, 0, 1, None);
    let pzone_card_count = duel.get_field_group_count(tp, LOCATION_PZONE, 0);
    let disable_chk = disable_chk(duel, tp, pzone_card_count);
    let mut op = Some(1);
    if disable_chk {
        op = duel.select_effect(tp, &[(place_chk, aux::stringid(ID, 2)), (disable_chk, aux::stringid(ID, 3))])?;
    }
    match op {
        Some(1) => {
            duel.hint(HINT_SELECTMSG, tp, HINTMSG_TOFIELD);
            let filter = aux::necro_valley_filter(plfilter);
            if let Some(sc) = duel
                .select_matching_card(tp, filter, tp, LOCATION_DECK | LOCATION_GRAVE, 0, 1, 1, None)?
                .get_first()
            {
                duel.place_field_spell(sc, tp)?;
            }
        }
        Some(2) => {
            duel.hint(HINT_SELECTMSG, tp, HINTMSG_NEGATE);
            let max = pzone_card_count as u8;
            let g = duel.select_matching_card(tp, |c, duel| c.is_negatable(duel), tp, 0, LOCATION_ONFIELD, 1, max, None)?;
            if g.is_empty() {
                return Ok(());
            }
            duel.hint_selection(g);
            let c = a.e.get_handler(duel);
            for sc in g {
                sc.negate_effects(duel, c, 0, true);
            }
        }
        _ => {}
    }
    Ok(())
}

fn posfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_ENNEACRAFT]) && c.is_can_turn_set(duel)
}

fn postg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.get_field_group_count(tp, LOCATION_PZONE, 0) > 0
            && duel.is_existing_matching_card(posfilter, tp, LOCATION_MZONE, 0, 1, None));
    }
    duel.set_operation_info(0, CATEGORY_POSITION, None, 1, tp, POS_FACEDOWN_DEFENSE as i32);
    Ok(false)
}

fn posop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    let max_count = duel.get_field_group_count(tp, LOCATION_PZONE, 0);
    if max_count == 0 {
        return Ok(());
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_POSCHANGE);
    let g = duel.select_matching_card(tp, posfilter, tp, LOCATION_MZONE, 0, 1, max_count as u8, None)?;
    if !g.is_empty() {
        duel.hint_selection(g);
        duel.change_position(g, POS_FACEDOWN_DEFENSE)?;
    }
    Ok(())
}
