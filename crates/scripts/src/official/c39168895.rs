// Berserk Gorilla
use engine::constants::MSG_ATTACK;

use crate::prelude::*;
use crate::testing::{Check, MZONE, OPPONENT};

const ID: u32 = 39168895;

// Copies on the field to attack, some of them placed in Defense Position and self-destroying, against the opponent's monsters.
pub const SUPPORT: &[(u32, u16)] = &[(ID, MZONE), (ID, MZONE), (14851496, MZONE | OPPONENT), (70781052, MZONE | OPPONENT)];
pub const CHECKS: &[Check] = &[Check::Message(MSG_ATTACK)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_code(duel, EFFECT_MUST_ATTACK);
    c.register_effect(duel, e1);
    let e3 = duel.create_effect(c);
    e3.set_type(duel, EFFECT_TYPE_SINGLE);
    e3.set_property(duel, EFFECT_FLAG_SINGLE_RANGE);
    e3.set_range(duel, LOCATION_MZONE);
    e3.set_code(duel, EFFECT_SELF_DESTROY);
    e3.set_condition(duel, |duel, a| a.e.get_handler(duel).is_defense_pos(duel));
    c.register_effect(duel, e3);
}
