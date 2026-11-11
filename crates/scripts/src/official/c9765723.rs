// Dark Contract with the Witch
use crate::prelude::*;
use crate::testing::{Check, HAND, MZONE, SZONE};

const ID: u32 = 9765723;

// Set on the field and in the hand, with D/D Lance Soldier (a Fiend) in the hand to discard and on the field.
pub const SUPPORT: &[(u32, u16)] = &[(ID, SZONE), (ID, SZONE), (ID, HAND), (67322708, HAND), (67322708, MZONE)];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_property(duel, EFFECT_FLAG_DAMAGE_STEP);
    e1.set_hint_timing(duel, 0, TIMINGS_CHECK_MONSTER_E | TIMING_DAMAGE_STEP);
    e1.set_condition(duel, aux::stat_change_damage_step_condition);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 0));
    e2.set_category(duel, CATEGORY_DESTROY);
    e2.set_type(duel, EFFECT_TYPE_QUICK_O);
    e2.set_range(duel, LOCATION_SZONE);
    e2.set_code(duel, EVENT_FREE_CHAIN);
    e2.set_hint_timing(duel, 0, TIMINGS_CHECK_MONSTER | TIMING_END_PHASE);
    e2.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e2.set_count_limit(duel, 1, ID);
    e2.set_cost(duel, descost);
    e2.set_target(duel, destg);
    e2.set_operation(duel, stock::destroy_first_target);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_type(duel, EFFECT_TYPE_FIELD);
    e3.set_range(duel, LOCATION_SZONE);
    e3.set_target_range(duel, LOCATION_MZONE, 0);
    e3.set_code(duel, EFFECT_UPDATE_ATTACK);
    e3.set_condition(duel, |duel, a| duel.get_turn_player() != a.e.get_handler_player(duel));
    e3.set_target_card(duel, |duel, _, c| c.is_race(duel, RACE_FIEND));
    e3.set_value(duel, 1000);
    c.register_effect(duel, e3);
    let e4 = duel.create_effect(c);
    e4.set_description(duel, aux::stringid(ID, 1));
    e4.set_category(duel, CATEGORY_DAMAGE);
    e4.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_F);
    e4.set_code(duel, EVENT_PHASE | PHASE_STANDBY);
    e4.set_range(duel, LOCATION_SZONE);
    e4.set_count_limit(duel, 1, 0);
    e4.set_condition(duel, |duel, a| duel.is_turn_player(a.tp));
    e4.set_target(duel, damtg);
    e4.set_operation(duel, stock::damage);
    c.register_effect(duel, e4);
}

fn cfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_DD, SET_DARK_CONTRACT]) && c.is_able_to_grave_as_cost(duel)
}

fn descost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(cfilter, tp, LOCATION_HAND, 0, 1, None));
    }
    duel.discard_hand(tp, cfilter, 1, 1, REASON_COST)?;
    Ok(false)
}

fn destg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_on_field(duel));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(aux::TRUE, tp, LOCATION_ONFIELD, LOCATION_ONFIELD, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_DESTROY);
    let g = duel.select_target(tp, aux::TRUE, tp, LOCATION_ONFIELD, LOCATION_ONFIELD, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_DESTROY, g, 1, 0, 0);
    Ok(false)
}

fn damtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(true);
    }
    duel.set_target_player(a.tp);
    duel.set_target_param(1000);
    duel.set_operation_info(0, CATEGORY_DAMAGE, None, 0, a.tp, 1000);
    Ok(false)
}
