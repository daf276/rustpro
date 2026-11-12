// Top Runner
use crate::prelude::*;
use crate::testing::{Check, MZONE, OPPONENT};
use engine::constants::MSG_ATTACK;

const ID: u32 = 53623827;

// Continuous effect only; on the field with Synchro Monsters (Magical Android) to attack Sangan of the opponent's.
pub const SUPPORT: &[(u32, u16)] = &[(ID, MZONE), (43385557, MZONE), (43385557, MZONE), (26202165, MZONE | OPPONENT)];
pub const CHECKS: &[Check] = &[Check::Message(MSG_ATTACK)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_target_range(duel, LOCATION_MZONE, 0);
    e1.set_code(duel, EFFECT_UPDATE_ATTACK);
    e1.set_target_card(duel, |duel, _, c| c.is_type(duel, TYPE_SYNCHRO));
    e1.set_value(duel, 600);
    c.register_effect(duel, e1);
}
