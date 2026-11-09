// Majestic Dragon
use engine::constants::MSG_ATTACK;

use crate::prelude::*;
use crate::testing::{Check, MZONE, OPPONENT};

const ID: u32 = 21159309;

// Only a restriction on Synchro Summons, with no effect to resolve: on the field, with an attacker against it.
pub const SUPPORT: &[(u32, u16)] = &[(ID, MZONE), (ID, MZONE | OPPONENT)];
pub const CHECKS: &[Check] = &[Check::Message(MSG_ATTACK)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_code(duel, EFFECT_CANNOT_BE_SYNCHRO_MATERIAL);
    e1.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE + EFFECT_FLAG_UNCOPYABLE);
    e1.set_value(duel, Value::Card(|duel, _, c| i32::from(!c.is_set_card(duel, &[SET_MAJESTIC]))));
    c.register_effect(duel, e1);
}
