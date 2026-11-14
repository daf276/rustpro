// Ryu-Ge Realm - Sea Spires
use super::c55154344::{activation, efftg, grant, is_realm_cost, to_deck_cost};
use crate::prelude::*;
use crate::testing::{Check, HAND, MZONE};

const ID: u32 = 28669235;

pub const LISTED_SERIES: &[u16] = &[SET_RYU_GE];
pub const LISTED_NAMES: &[u32] = &[ID];
// Copies in the hand to activate, with Kairo Ryu-Ge Emva (Level 10 Sea Serpent) on the field to be granted the effect.
pub const SUPPORT: &[(u32, u16)] = &[(ID, HAND), (ID, HAND), (20904475, MZONE), (20904475, MZONE)];
pub const CHECKS: &[Check] = &[Check::ResolvesGranted(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    activation::<ID>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_code(duel, EFFECT_INDESTRUCTABLE_BATTLE);
    e1.set_range(duel, LOCATION_SZONE);
    e1.set_target_range(duel, LOCATION_MZONE, 0);
    e1.set_target_card(duel, |duel, _, c| c.is_set_card(duel, &[SET_RYU_GE]));
    e1.set_value(duel, 1);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 0));
    e2.set_category(duel, CATEGORY_TOHAND);
    e2.set_type(duel, EFFECT_TYPE_QUICK_O);
    e2.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e2.set_code(duel, EVENT_FREE_CHAIN);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_count_limit(duel, 1, 0);
    e2.set_hint_timing(duel, 0, TIMING_STANDBY_PHASE | TIMING_MAIN_END | TIMINGS_CHECK_MONSTER_E);
    e2.set_condition(duel, |duel, a| duel.is_turn_player(1 - a.tp));
    e2.set_cost(duel, thcost);
    e2.set_target(duel, thtg);
    e2.set_operation(duel, stock::first_target_to_hand);
    grant(duel, c, e2, efftg::<RACE_SEASERPENT>);
}

fn thcost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    to_deck_cost(duel, a, chk, |c, duel| {
        is_realm_cost(c, duel) && duel.is_existing_target(CardId::is_able_to_hand, 0, LOCATION_ONFIELD, LOCATION_ONFIELD, 1, Some(c.into()))
    })
}

fn thtg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_on_field(duel) && chkc.is_able_to_hand(duel));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(CardId::is_able_to_hand, tp, LOCATION_ONFIELD, LOCATION_ONFIELD, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_RTOHAND);
    let g = duel.select_target(tp, CardId::is_able_to_hand, tp, LOCATION_ONFIELD, LOCATION_ONFIELD, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_TOHAND, g, 1, tp, 0);
    Ok(false)
}
