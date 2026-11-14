// Darklord Rebellion
use crate::prelude::*;
use crate::testing::{Check, HAND, MZONE, SZONE};

const ID: u32 = 50501121;

pub const LISTED_SERIES: &[u16] = &[SET_DARKLORD];

const DARKLORD_GULGOLET: u32 = 84031359;
const DARKLORD_DJEHUTY: u32 = 10426067;

// Set, with Darklords in the hand and on the field to send.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, SZONE),
    (DARKLORD_GULGOLET, HAND),
    (DARKLORD_DJEHUTY, HAND),
    (DARKLORD_GULGOLET, MZONE),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(0)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_category(duel, CATEGORY_DESTROY);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_count_limit_full(duel, 1, ID, EFFECT_COUNT_CODE_OATH);
    e1.set_cost(duel, cost);
    e1.set_target(duel, target);
    e1.set_operation(duel, activate);
    c.register_effect(duel, e1);
}

fn costfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_DARKLORD])
        && c.is_monster(duel)
        && (c.is_location(duel, LOCATION_HAND) || c.is_faceup(duel))
        && c.is_able_to_grave_as_cost(duel)
        && duel.is_existing_matching_card(aux::TRUE, 0, LOCATION_ONFIELD, LOCATION_ONFIELD, 1, Some(c.into()))
}

fn cost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    stock::send_to_grave_cost(duel, a, chk, costfilter, LOCATION_HAND | LOCATION_MZONE)
}

fn exception(duel: &Duel, e: EffectId) -> Option<CardSet> {
    e.is_has_type(duel, EFFECT_TYPE_ACTIVATE).then(|| e.get_handler(duel).into())
}

fn target(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let g = duel.get_matching_group(aux::TRUE, a.tp, LOCATION_ONFIELD, LOCATION_ONFIELD, exception(duel, a.e));
    if chk == 0 {
        return Ok(!g.is_empty());
    }
    duel.set_operation_info(0, CATEGORY_DESTROY, Some(g), 1, 0, 0);
    Ok(false)
}

fn activate(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    let exc = exception(duel, a.e);
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_DESTROY);
    let g = duel.select_matching_card(tp, aux::TRUE, tp, LOCATION_ONFIELD, LOCATION_ONFIELD, 1, 1, exc)?;
    if !g.is_empty() {
        duel.hint_selection(g);
        duel.destroy(g, REASON_EFFECT)?;
    }
    Ok(())
}
