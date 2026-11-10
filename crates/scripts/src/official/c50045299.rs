// Dragon Capture Jar
use crate::prelude::*;
use crate::testing::{Check, HAND, MZONE, OPPONENT};

const ID: u32 = 50045299;

// In the hand, with a Dragon (Petit Dragon, a Normal Monster) on each side.
pub const SUPPORT: &[(u32, u16)] = &[(ID, HAND), (75356564, MZONE), (75356564, MZONE | OPPONENT)];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(0)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_FIELD);
    e2.set_code(duel, EFFECT_SET_POSITION);
    e2.set_range(duel, LOCATION_SZONE);
    e2.set_target_card(duel, |duel, _, c| c.is_race(duel, RACE_DRAGON));
    e2.set_target_range(duel, LOCATION_MZONE, LOCATION_MZONE);
    e2.set_value(duel, POS_FACEUP_DEFENSE as i32);
    c.register_effect(duel, e2);
    let e3 = e2.clone_effect(duel);
    e3.set_code(duel, EFFECT_CANNOT_CHANGE_POSITION);
    c.register_effect(duel, e3);
}
