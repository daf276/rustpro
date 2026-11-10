// Mountain
use crate::prelude::*;
use crate::testing::{Check, HAND, MZONE};

const ID: u32 = 50913601;

// A Dragon Normal Monster, Petit Dragon, on the field for it to raise.
pub const SUPPORT: &[(u32, u16)] = &[(ID, HAND), (75356564, MZONE)];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(0)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_FIELD);
    e2.set_code(duel, EFFECT_UPDATE_ATTACK);
    e2.set_range(duel, LOCATION_FZONE);
    e2.set_target_range(duel, LOCATION_MZONE, LOCATION_MZONE);
    e2.set_target_card(duel, |duel, _, c| c.is_race(duel, RACE_DRAGON | RACE_WINGEDBEAST | RACE_THUNDER));
    e2.set_value(duel, 200);
    c.register_effect(duel, e2);
    let e3 = e2.clone_effect(duel);
    e3.set_code(duel, EFFECT_UPDATE_DEFENSE);
    c.register_effect(duel, e3);
}
