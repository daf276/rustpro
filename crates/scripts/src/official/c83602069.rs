// Comrade Swordsman of Landstar
use crate::prelude::*;
use crate::testing::{Check, MZONE, OPPONENT};
use engine::constants::MSG_ATTACK;

const ID: u32 = 83602069;

// Continuous effect only; on the field with Warriors (Kojikocy) and Sangan of the opponent's to attack, and attacked.
pub const SUPPORT: &[(u32, u16)] = &[(ID, MZONE), (1184620, MZONE), (1184620, MZONE), (26202165, MZONE | OPPONENT)];
pub const CHECKS: &[Check] = &[Check::Message(MSG_ATTACK)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_target_range(duel, LOCATION_MZONE, 0);
    e1.set_code(duel, EFFECT_UPDATE_ATTACK);
    e1.set_target_card(duel, |duel, _, c| c.is_race(duel, RACE_WARRIOR));
    e1.set_value(duel, 400);
    c.register_effect(duel, e1);
}
