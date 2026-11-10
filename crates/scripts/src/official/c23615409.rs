// Insect Barrier
use engine::constants::MSG_ATTACK;

use crate::prelude::*;
use crate::testing::{Check, HAND, MZONE, OPPONENT, SZONE};

const ID: u32 = 23615409;

// In the hand to be activated, with the opponent's Insects, and an attacker of the opponent that stays able to attack.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, SZONE),
    (3134241, MZONE | OPPONENT),
    (4148264, MZONE | OPPONENT),
    (13597785, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[Check::Resolves, Check::Message(MSG_ATTACK)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_FIELD);
    e2.set_code(duel, EFFECT_CANNOT_ATTACK_ANNOUNCE);
    e2.set_range(duel, LOCATION_SZONE);
    e2.set_target_range(duel, 0, LOCATION_MZONE);
    e2.set_target_card(duel, |duel, _, c| c.is_race(duel, RACE_INSECT));
    c.register_effect(duel, e2);
}
