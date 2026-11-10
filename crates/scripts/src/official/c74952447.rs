// Testudo erat Numen
use engine::constants::MSG_ATTACK;

use crate::prelude::*;
use crate::testing::{Check, MZONE, OPPONENT};

const ID: u32 = 74952447;

// On the field with an opponent's Sangan to attack.
pub const SUPPORT: &[(u32, u16)] = &[(ID, MZONE), (26202165, MZONE | OPPONENT)];
// It has only a continuous restriction.
pub const CHECKS: &[Check] = &[Check::Message(MSG_ATTACK)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_code(duel, EFFECT_CANNOT_SPECIAL_SUMMON);
    e1.set_property(duel, EFFECT_FLAG_PLAYER_TARGET);
    e1.set_target_range(duel, 1, 1);
    e1.set_target_card(duel, |duel, _, c| c.is_attack_above(duel, 1800));
    c.register_effect(duel, e1);
}
