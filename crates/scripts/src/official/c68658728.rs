// Little Chimera
use crate::prelude::*;
use crate::testing::{Check, HAND, MZONE};
use engine::constants::MSG_ATTACK;

const ID: u32 = 68658728;

// Continuous effects only; a FIRE and a WATER Normal Monster (Darkfire Soldier #1, Island Turtle) on the field to attack.
pub const SUPPORT: &[(u32, u16)] = &[(ID, HAND), (5388481, MZONE), (4042268, MZONE), (4042268, MZONE)];
pub const CHECKS: &[Check] = &[Check::Message(MSG_ATTACK)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_target_range(duel, LOCATION_MZONE, LOCATION_MZONE);
    e1.set_code(duel, EFFECT_UPDATE_ATTACK);
    e1.set_target_card(duel, |duel, _, c| c.is_attribute(duel, ATTRIBUTE_FIRE));
    e1.set_value(duel, 500);
    c.register_effect(duel, e1);
    let e2 = e1.clone_effect(duel);
    e2.set_target_card(duel, |duel, _, c| c.is_attribute(duel, ATTRIBUTE_WATER));
    e2.set_value(duel, -400);
    c.register_effect(duel, e2);
}
