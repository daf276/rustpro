// Invader of Darkness
use engine::constants::MSG_ATTACK;

use crate::prelude::*;
use crate::testing::{Check, HAND, MZONE, OPPONENT, SZONE};

const ID: u32 = 56647086;

// On the field, against Mystical Space Typhoons of the opponent's it locks.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (ID, MZONE),
    (5318639, HAND | OPPONENT),
    (5318639, SZONE | OPPONENT),
    (5318639, SZONE | OPPONENT),
];
// Only a continuous effect: it checks that the card fights.
pub const CHECKS: &[Check] = &[Check::Message(MSG_ATTACK)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_property(duel, EFFECT_FLAG_PLAYER_TARGET);
    e1.set_code(duel, EFFECT_CANNOT_ACTIVATE);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_target_range(duel, 0, 1);
    e1.set_value(
        duel,
        Value::EffectPlayer(|duel, _, re, _| re.get_handler(duel).is_quick_play_spell(duel) && re.is_has_type(duel, EFFECT_TYPE_ACTIVATE)),
    );
    c.register_effect(duel, e1);
}
