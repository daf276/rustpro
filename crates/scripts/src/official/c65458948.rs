// Toon Mermaid
use engine::constants::MSG_ATTACK;

use super::c53183600::{atcon, atklimit, atlimit, atop, dircon, sdescon, sdesop, toon_world_on_field};
use crate::prelude::*;
use crate::testing::{Check, HAND, MZONE, OPPONENT, SZONE};

const ID: u32 = 65458948;

pub const LISTED_NAMES: &[u32] = &[CARD_TOON_WORLD];

// In the hand, Special Summoned with Toon World on the field, to attack the opponent's monsters.
pub const SUPPORT: &[(u32, u16)] = &[(ID, HAND), (CARD_TOON_WORLD, SZONE), (26202165, MZONE | OPPONENT)];
// It has no effect to resolve.
pub const CHECKS: &[Check] = &[Check::SpecialSummoned, Check::Message(MSG_ATTACK)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_FIELD);
    e2.set_code(duel, EFFECT_SPSUMMON_PROC);
    e2.set_property(duel, EFFECT_FLAG_UNCOPYABLE);
    e2.set_range(duel, LOCATION_HAND);
    e2.set_condition(duel, spcon);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_CONTINUOUS);
    e3.set_range(duel, LOCATION_MZONE);
    e3.set_code(duel, EVENT_LEAVE_FIELD);
    e3.set_condition(duel, sdescon);
    e3.set_operation(duel, sdesop);
    c.register_effect(duel, e3);
    let e4 = duel.create_effect(c);
    e4.set_type(duel, EFFECT_TYPE_SINGLE);
    e4.set_code(duel, EFFECT_DIRECT_ATTACK);
    e4.set_condition(duel, dircon);
    c.register_effect(duel, e4);
    let e5 = duel.create_effect(c);
    e5.set_type(duel, EFFECT_TYPE_SINGLE);
    e5.set_code(duel, EFFECT_CANNOT_SELECT_BATTLE_TARGET);
    e5.set_condition(duel, atcon);
    e5.set_value(duel, Value::Card(|duel, _, c| i32::from(atlimit(duel, c))));
    c.register_effect(duel, e5);
    let e6 = duel.create_effect(c);
    e6.set_type(duel, EFFECT_TYPE_SINGLE);
    e6.set_code(duel, EFFECT_CANNOT_DIRECT_ATTACK);
    e6.set_condition(duel, atcon);
    c.register_effect(duel, e6);
    let e7 = duel.create_effect(c);
    e7.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_CONTINUOUS);
    e7.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e7.set_operation(duel, atklimit);
    c.register_effect(duel, e7);
    let e8 = duel.create_effect(c);
    e8.set_type(duel, EFFECT_TYPE_SINGLE);
    e8.set_code(duel, EFFECT_ATTACK_COST);
    e8.set_cost_condition(duel, |duel, _, _, tp, _| duel.check_lp_cost(tp, 500));
    e8.set_operation(duel, atop);
    c.register_effect(duel, e8);
}

fn spcon(duel: &Duel, a: &Args) -> bool {
    let Some(c) = a.c else { return true };
    let tp = c.get_controler(duel);
    duel.get_location_count(tp, LOCATION_MZONE) > 0 && toon_world_on_field(duel, tp)
}
