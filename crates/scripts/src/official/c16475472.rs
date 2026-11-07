// Lesser Fiend
use crate::prelude::*;
use crate::testing::{Check, MZONE};
use engine::constants::MSG_ATTACK;

const ID: u32 = 16475472;

pub const SUPPORT: &[(u32, u16)] = &[(ID, MZONE)];
// The banishing shows on the opponent's monster, not on this card.
pub const CHECKS: &[Check] = &[Check::Message(MSG_ATTACK)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_property(duel, EFFECT_FLAG_SINGLE_RANGE);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_code(duel, EFFECT_BATTLE_DESTROY_REDIRECT);
    e1.set_value(duel, LOCATION_REMOVED as i32);
    c.register_effect(duel, e1);
}
