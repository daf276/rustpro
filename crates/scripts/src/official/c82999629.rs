// Umiiruka
use crate::prelude::*;
use crate::testing::{HAND, MZONE};

const ID: u32 = 82999629;

// A Field Spell, in the hand to be activated, with WATER monsters on the field.
pub const SUPPORT: &[(u32, u16)] = &[(ID, HAND), (18914778, MZONE), (58538870, MZONE)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_FIELD);
    e2.set_range(duel, LOCATION_FZONE);
    e2.set_target_range(duel, LOCATION_MZONE, LOCATION_MZONE);
    e2.set_code(duel, EFFECT_UPDATE_ATTACK);
    e2.set_target_card(duel, |duel, _, c| c.is_attribute(duel, ATTRIBUTE_WATER));
    e2.set_value(duel, 500);
    c.register_effect(duel, e2);
    let e3 = e2.clone_effect(duel);
    e3.set_code(duel, EFFECT_UPDATE_DEFENSE);
    e3.set_value(duel, -400);
    c.register_effect(duel, e3);
}
