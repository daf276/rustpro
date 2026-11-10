// Mind Crush
use crate::prelude::*;
use crate::testing::{Check, SZONE};

const ID: u32 = 15800838;

// Set, to activate with cards in both hands.
pub const SUPPORT: &[(u32, u16)] = &[(ID, SZONE)];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(0)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_hint_timing(duel, 0, TIMING_TOHAND);
    e1.set_category(duel, CATEGORY_HANDES);
    e1.set_target(duel, target);
    e1.set_operation(duel, operation);
    c.register_effect(duel, e1);
}

fn target(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.get_field_group_count(tp, 0, LOCATION_HAND) > 0
            && duel.is_existing_matching_card(aux::TRUE, tp, LOCATION_HAND, 0, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_CODE);
    let announce_filter = [
        u64::from(TYPE_EXTRA),
        OPCODE_ISTYPE,
        u64::from(TYPE_MONSTER),
        OPCODE_ISTYPE,
        OPCODE_AND,
        OPCODE_NOT,
    ];
    let ac = duel.announce_card(tp, &announce_filter)?;
    duel.set_target_param(ac);
    duel.set_operation_info(0, CATEGORY_ANNOUNCE, None, 0, tp, ANNOUNCE_CARD_FILTER as i32);
    Ok(false)
}

fn operation(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    let ac: u32 = duel.get_chain_info(0, CHAININFO_TARGET_PARAM);
    let g = duel.get_matching_group(|c, duel| c.is_code(duel, ac), tp, 0, LOCATION_HAND, None);
    if !g.is_empty() {
        duel.send_to_grave(g, REASON_EFFECT | REASON_DISCARD)?;
    } else {
        let dg = duel.get_field_group(tp, LOCATION_HAND, 0).random_select(duel, tp, 1);
        duel.send_to_grave(dg, REASON_EFFECT | REASON_DISCARD)?;
    }
    Ok(())
}
