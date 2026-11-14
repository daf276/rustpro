// Lancephorhynchus
use crate::prelude::*;
use crate::testing::{Check, MZONE, OPPONENT, PZONE};
use engine::constants::MSG_ATTACK;

const ID: u32 = 48940337;

// In the Pendulum Zone, with Summoned Skulls (Normal Monsters) to attack the opponent's Sangans.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, PZONE),
    (70781052, MZONE),
    (70781052, MZONE),
    (26202165, MZONE | OPPONENT),
    (26202165, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[Check::Message(MSG_ATTACK)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    Pendulum::add_procedure(duel, c);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_FIELD);
    e2.set_code(duel, EFFECT_PIERCE);
    e2.set_range(duel, LOCATION_PZONE);
    e2.set_target_range(duel, LOCATION_MZONE, 0);
    e2.set_target_card(duel, |duel, _, c| c.is_type(duel, TYPE_NORMAL));
    c.register_effect(duel, e2);
}
