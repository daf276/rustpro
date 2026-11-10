// Enraged Battle Ox
use engine::constants::MSG_ATTACK;

use crate::prelude::*;
use crate::testing::{Check, MZONE, OPPONENT};

const ID: u32 = 76909279;

// No activated effect: Battle Ox on the field attack the opponent's Mokey Mokey.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (5053103, MZONE),
    (5053103, MZONE),
    (27288416, MZONE | OPPONENT),
    (27288416, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[Check::Message(MSG_ATTACK)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_code(duel, EFFECT_PIERCE);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_target_range(duel, LOCATION_MZONE, 0);
    e1.set_target_card(duel, |duel, _, c| c.is_race(duel, RACES_BEAST_BWARRIOR_WINGB));
    c.register_effect(duel, e1);
}
