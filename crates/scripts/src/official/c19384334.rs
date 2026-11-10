// Molten Destruction
use crate::prelude::*;
use crate::testing::{HAND, MZONE};

const ID: u32 = 19384334;

// A Field Spell, in the hand to be activated, with FIRE monsters on the field.
pub const SUPPORT: &[(u32, u16)] = &[(ID, HAND), (5464695, MZONE), (5388481, MZONE)];

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
    e2.set_target_card(duel, |duel, _, c| c.is_attribute(duel, ATTRIBUTE_FIRE));
    e2.set_value(duel, 500);
    c.register_effect(duel, e2);
    let e3 = e2.clone_effect(duel);
    e3.set_code(duel, EFFECT_UPDATE_DEFENSE);
    e3.set_value(duel, -400);
    c.register_effect(duel, e3);
}
