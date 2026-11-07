// Elemental HERO Neos Alius
use crate::prelude::*;
use crate::testing::{Check, MZONE};

pub const LISTED_NAMES: &[u32] = &[CARD_NEOS];

// On the field twice, to be Normal Summoned again.
pub const SUPPORT: &[(u32, u16)] = &[(69884162, MZONE), (69884162, MZONE)];
// The second Normal Summon's hint.
pub const CHECKS: &[Check] = &[Check::CardHint(64)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    Gemini::add_procedure(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_property(duel, EFFECT_FLAG_SINGLE_RANGE);
    e1.set_code(duel, EFFECT_CHANGE_CODE);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_condition(duel, Gemini::effect_status_condition);
    e1.set_value(duel, CARD_NEOS as i32);
    c.register_effect(duel, e1);
}
