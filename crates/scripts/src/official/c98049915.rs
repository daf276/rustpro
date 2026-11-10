// Mystic Lamp
use engine::constants::MSG_ATTACK;

use crate::prelude::*;
use crate::testing::{Check, MZONE};

const ID: u32 = 98049915;

pub const SUPPORT: &[(u32, u16)] = &[(ID, MZONE)];
pub const CHECKS: &[Check] = &[Check::Message(MSG_ATTACK)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_code(duel, EFFECT_DIRECT_ATTACK);
    c.register_effect(duel, e1);
}
