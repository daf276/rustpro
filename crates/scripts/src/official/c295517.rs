// A Legendary Ocean
use crate::prelude::*;
use crate::testing::{Check, HAND, MZONE};

const ID: u32 = 295517;

// In the hand, with a WATER monster (Water Spirit) on the field and in the hand.
pub const SUPPORT: &[(u32, u16)] = &[(ID, HAND), (487395, MZONE), (487395, HAND)];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(0)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_FIELD);
    e2.set_code(duel, EFFECT_UPDATE_LEVEL);
    e2.set_range(duel, LOCATION_FZONE);
    e2.set_target_range(duel, LOCATION_HAND | LOCATION_MZONE, LOCATION_HAND | LOCATION_MZONE);
    e2.set_target_card(duel, |duel, _, c| c.is_attribute(duel, ATTRIBUTE_WATER));
    e2.set_value(duel, -1);
    c.register_effect(duel, e2);
    let e3 = e2.clone_effect(duel);
    e3.set_code(duel, EFFECT_UPDATE_ATTACK);
    e3.set_target_range(duel, LOCATION_MZONE, LOCATION_MZONE);
    e3.set_value(duel, 200);
    c.register_effect(duel, e3);
    let e4 = e3.clone_effect(duel);
    e4.set_code(duel, EFFECT_UPDATE_DEFENSE);
    c.register_effect(duel, e4);
}
