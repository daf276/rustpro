// Dragunity Brandistock
use crate::prelude::*;
use crate::testing::{Check, DECK, HAND, MZONE};
use engine::constants::MSG_ATTACK;

const ID: u32 = 54455664;

// Dragunity Falchion in hand to equip a copy from the Deck to a copy on the field, which attacks.
pub const SUPPORT: &[(u32, u16)] = &[(ID, MZONE), (ID, MZONE), (ID, DECK), (ID, DECK), (8039562, HAND), (8039562, HAND)];
pub const CHECKS: &[Check] = &[Check::Message(MSG_ATTACK)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_EQUIP);
    e1.set_code(duel, EFFECT_EXTRA_ATTACK);
    e1.set_value(duel, 1);
    c.register_effect(duel, e1);
}
