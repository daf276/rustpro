// Volcanic Rimfire
use crate::prelude::*;
use crate::testing::{Check, DECK, GRAVE, HAND, MZONE, OPPONENT, SZONE};

const ID: u32 = 59805313;

pub const LISTED_SERIES: &[u16] = &[SET_VOLCANIC, SET_BLAZE_ACCELERATOR];
pub const LISTED_NAMES: &[u32] = &[ID];

// In hand and on the field to reach the Graveyard through Blaze Accelerator or battle, with Volcanic Shell in the Deck to
// send and Blaze Accelerators in the Graveyard and Deck to banish and place.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, MZONE),
    (69537999, SZONE),
    (69537999, GRAVE),
    (69537999, DECK),
    (33365932, DECK),
    (33365932, DECK),
    (13597785, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 0));
    e3.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e3.set_property(duel, EFFECT_FLAG_DELAY);
    e3.set_code(duel, EVENT_TO_GRAVE);
    e3.set_target(duel, efftg);
    e3.set_operation(duel, effop);
    c.register_effect(duel, e3);
}

fn tgfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_VOLCANIC]) && c.is_monster(duel) && !c.is_code(duel, ID) && c.is_able_to_grave(duel)
}

fn rmfilter(c: CardId, duel: &Duel, tp: u8, ft: i32) -> bool {
    (ft > 0 || c.is_location(duel, LOCATION_STZONE))
        && c.is_set_card(duel, &[SET_BLAZE_ACCELERATOR])
        && c.is_able_to_remove(duel)
        && c.is_faceup(duel)
        && duel.is_existing_matching_card(|c, duel| plfilter(c, duel, tp), tp, LOCATION_HAND | LOCATION_DECK, 0, 1, None)
}

fn plfilter(c: CardId, duel: &Duel, tp: u8) -> bool {
    c.is_set_card(duel, &[SET_BLAZE_ACCELERATOR])
        && c.is_continuous_spell_trap(duel)
        && !c.is_forbidden(duel)
        && c.check_unique_on_field(duel, tp)
}

fn efftg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    let ft = duel.get_location_count(tp, LOCATION_SZONE);
    let b1 = !duel.has_flag_effect(tp, ID)
        && c.is_able_to_remove(duel)
        && duel.is_existing_matching_card(tgfilter, tp, LOCATION_DECK, 0, 1, None);
    let b2 = !duel.has_flag_effect(tp, ID + 1)
        && duel.is_existing_matching_card(
            |c, duel| rmfilter(c, duel, tp, ft),
            tp,
            LOCATION_ONFIELD | LOCATION_GRAVE,
            0,
            1,
            None,
        );
    if chk == 0 {
        return Ok(b1 || b2);
    }
    let op = duel.select_effect(tp, &[(b1, aux::stringid(ID, 1)), (b2, aux::stringid(ID, 2))])?;
    e.set_label(duel, op.map_or(0, |op| op as i64));
    if op == Some(1) {
        duel.register_flag_effect(tp, ID, RESET_PHASE | PHASE_END, 0, 1);
        e.set_category(duel, CATEGORY_REMOVE + CATEGORY_TOGRAVE);
        duel.set_operation_info(0, CATEGORY_REMOVE, Some(c.into()), 1, tp, LOCATION_GRAVE as i32);
        duel.set_operation_info(0, CATEGORY_TOGRAVE, None, 1, tp, LOCATION_DECK as i32);
    } else if op == Some(2) {
        duel.register_flag_effect(tp, ID + 1, RESET_PHASE | PHASE_END, 0, 1);
        e.set_category(duel, CATEGORY_REMOVE);
        duel.set_operation_info(0, CATEGORY_REMOVE, None, 1, tp, (LOCATION_ONFIELD | LOCATION_GRAVE) as i32);
    }
    Ok(false)
}

fn effop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    match e.get_label(duel) {
        1 => {
            let c = e.get_handler(duel);
            if !c.is_relate_to_effect(duel, e) || duel.remove(c, POS_FACEUP, REASON_EFFECT)? == 0 {
                return Ok(());
            }
            stock::send_to_grave(duel, a, tgfilter, LOCATION_DECK)?;
        }
        2 => {
            let ft = duel.get_location_count(tp, LOCATION_SZONE);
            duel.hint(HINT_SELECTMSG, tp, HINTMSG_REMOVE);
            let g = duel.select_matching_card(
                tp,
                |c, duel| rmfilter(c, duel, tp, ft),
                tp,
                LOCATION_ONFIELD | LOCATION_GRAVE,
                0,
                1,
                1,
                None,
            )?;
            if g.is_empty() || duel.remove(g, POS_FACEUP, REASON_EFFECT)? == 0 {
                return Ok(());
            }
            duel.hint(HINT_SELECTMSG, tp, HINTMSG_TOFIELD);
            let plc = duel.select_matching_card(
                tp,
                |c, duel| plfilter(c, duel, tp),
                tp,
                LOCATION_HAND | LOCATION_DECK,
                0,
                1,
                1,
                None,
            )?;
            let Some(plc) = plc.get_first() else {
                unported!(duel, true);
                return Ok(());
            };
            duel.move_to_field(plc, tp, tp, LOCATION_SZONE, POS_FACEUP, true)?;
        }
        _ => {}
    }
    Ok(())
}
