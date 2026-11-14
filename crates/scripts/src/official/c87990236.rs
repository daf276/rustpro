// Darklord Enchantment
use crate::prelude::*;
use crate::testing::{Check, HAND, SZONE};

const ID: u32 = 87990236;

pub const LISTED_SERIES: &[u16] = &[SET_DARKLORD];

const DARKLORD_GULGOLET: u32 = 84031359;
const DARKLORD_DJEHUTY: u32 = 10426067;

// Set, with Darklords in the hand to send.
pub const SUPPORT: &[(u32, u16)] = &[(ID, SZONE), (DARKLORD_GULGOLET, HAND), (DARKLORD_DJEHUTY, HAND)];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_CONTROL);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_count_limit_full(duel, 1, ID, EFFECT_COUNT_CODE_OATH);
    e1.set_cost(duel, cost);
    e1.set_target(duel, target);
    e1.set_operation(duel, activate);
    c.register_effect(duel, e1);
}

fn costfilter(c: CardId, duel: &Duel, tp: u8) -> bool {
    c.is_set_card(duel, &[SET_DARKLORD])
        && c.is_monster(duel)
        && (c.is_location(duel, LOCATION_HAND) || c.is_faceup(duel))
        && c.is_able_to_grave_as_cost(duel)
        && duel.get_mzone_count_full(tp, Some(c.into()), tp, LOCATION_REASON_CONTROL, 0xff) > 0
}

fn cost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let tp = a.tp;
    a.e.set_label(duel, 100);
    let filter = |c, duel: &Duel| costfilter(c, duel, tp);
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(filter, tp, LOCATION_HAND | LOCATION_MZONE, 0, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TOGRAVE);
    let g = duel.select_matching_card(tp, filter, tp, LOCATION_HAND | LOCATION_MZONE, 0, 1, 1, None)?;
    duel.send_to_grave(g, REASON_COST)?;
    Ok(false)
}

fn filter(c: CardId, duel: &Duel, ignzn: bool) -> bool {
    c.is_controler_can_be_changed_full(duel, ignzn, 0xff) && c.is_faceup(duel)
}

fn target(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        let ignzn = a.e.get_label(duel) == 100;
        a.e.set_label(duel, 0);
        return Ok(duel.is_existing_matching_card(|c, duel| filter(c, duel, ignzn), a.tp, 0, LOCATION_MZONE, 1, None));
    }
    duel.set_operation_info(0, CATEGORY_CONTROL, None, 1, 0, 0);
    Ok(false)
}

fn activate(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_CONTROL);
    let g = duel.select_matching_card(tp, |c, duel| filter(c, duel, false), tp, 0, LOCATION_MZONE, 1, 1, None)?;
    if let Some(tc) = g.get_first() {
        let reason_player = duel.get_reason_player();
        duel.get_control_full(tc, tp, PHASE_END as u16, 1, 0xff, reason_player)?;
    }
    Ok(())
}
