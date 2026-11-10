// Gyaku-Gire Panda
use engine::constants::MSG_ATTACK;

use crate::prelude::*;
use crate::testing::{Check, MZONE, OPPONENT};

const ID: u32 = 9817927;

// No activated effect: it attacks the opponent's monsters.
pub const SUPPORT: &[(u32, u16)] = &[(ID, MZONE), (43989315, MZONE | OPPONENT), (43989315, MZONE | OPPONENT)];
pub const CHECKS: &[Check] = &[Check::Message(MSG_ATTACK)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_property(duel, EFFECT_FLAG_SINGLE_RANGE);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_code(duel, EFFECT_UPDATE_ATTACK);
    e1.set_value(duel, Value::Card(val));
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_SINGLE);
    e2.set_code(duel, EFFECT_PIERCE);
    c.register_effect(duel, e2);
}

fn val(duel: &Duel, _: EffectId, c: CardId) -> i32 {
    (duel.get_field_group_count(c.get_controler(duel), 0, LOCATION_MZONE) * 500) as i32
}
