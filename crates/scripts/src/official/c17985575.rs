// Lord of D.
use engine::constants::MSG_ATTACK;

use crate::prelude::*;
use crate::testing::{Check, MZONE, OPPONENT};

const ID: u32 = 17985575;

// No activated effect: it attacks Sangan of the opponent's.
pub const SUPPORT: &[(u32, u16)] = &[(ID, MZONE), (26202165, MZONE | OPPONENT)];
pub const CHECKS: &[Check] = &[Check::Message(MSG_ATTACK)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_code(duel, EFFECT_CANNOT_BE_EFFECT_TARGET);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_property(duel, EFFECT_FLAG_IGNORE_IMMUNE);
    e1.set_target_range(duel, LOCATION_MZONE, LOCATION_MZONE);
    e1.set_target_card(duel, |duel, _, c| c.is_race(duel, RACE_DRAGON));
    e1.set_value(duel, 1);
    c.register_effect(duel, e1);
}
