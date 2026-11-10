// Command Knight
use crate::prelude::*;
use crate::testing::{Check, MZONE, OPPONENT};
use engine::constants::MSG_ATTACK;

const ID: u32 = 10375182;

// Continuous effects only; on the field with a Warrior (Kojikocy) and Sangan of the opponent's to attack, and attacked.
pub const SUPPORT: &[(u32, u16)] = &[(ID, MZONE), (1184620, MZONE), (1184620, MZONE), (26202165, MZONE | OPPONENT)];
pub const CHECKS: &[Check] = &[Check::Message(MSG_ATTACK)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_property(duel, EFFECT_FLAG_SINGLE_RANGE);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_code(duel, EFFECT_CANNOT_BE_BATTLE_TARGET);
    e1.set_condition(duel, |duel, a| {
        duel.get_field_group_count(a.e.get_handler_player(duel), LOCATION_MZONE, 0) > 1
    });
    e1.set_value(duel, Value::Card(aux::imval1));
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_FIELD);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_target_range(duel, LOCATION_MZONE, 0);
    e2.set_code(duel, EFFECT_UPDATE_ATTACK);
    e2.set_target_card(duel, |duel, _, c| c.is_race(duel, RACE_WARRIOR));
    e2.set_value(duel, 400);
    c.register_effect(duel, e2);
}
