// Orcustrated Babel
use crate::prelude::*;
use crate::testing::{Check, GRAVE, HAND};

const ID: u32 = 90351981;

pub const SUPPORT: &[(u32, u16)] = &[(ID, HAND), (ID, GRAVE), (ID, GRAVE)];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(0), Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_FIELD);
    e2.set_property(duel, EFFECT_FLAG_PLAYER_TARGET);
    e2.set_code(duel, ID);
    e2.set_range(duel, LOCATION_FZONE);
    e2.set_target_range(duel, 1, 0);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 0));
    e3.set_category(duel, CATEGORY_TOHAND);
    e3.set_type(duel, EFFECT_TYPE_IGNITION);
    e3.set_range(duel, LOCATION_GRAVE);
    e3.set_condition(duel, aux::exccon);
    e3.set_cost(duel, thcost);
    e3.set_target(duel, thtg);
    e3.set_operation(duel, stock::self_to_hand);
    c.register_effect(duel, e3);
}

fn thcost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(CardId::is_able_to_grave_as_cost, tp, LOCATION_HAND, 0, 1, None));
    }
    duel.discard_hand(tp, CardId::is_able_to_grave_as_cost, 1, 1, REASON_COST)?;
    Ok(false)
}

fn thtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let c = a.e.get_handler(duel);
    if chk == 0 {
        return Ok(c.is_able_to_hand(duel));
    }
    duel.set_operation_info(0, CATEGORY_TOHAND, Some(c.into()), 1, 0, 0);
    duel.set_operation_info(0, CATEGORY_LEAVE_GRAVE, Some(c.into()), 1, 0, 0);
    Ok(false)
}
