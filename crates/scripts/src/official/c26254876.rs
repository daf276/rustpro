// Gemini Lancer
use crate::prelude::*;
use crate::testing::{Check, MZONE};

// On the field twice, to be Normal Summoned again.
pub const SUPPORT: &[(u32, u16)] = &[(26254876, MZONE), (26254876, MZONE)];
// The second Normal Summon's hint.
pub const CHECKS: &[Check] = &[Check::CardHint(64)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    Gemini::add_procedure(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_code(duel, EFFECT_PIERCE);
    e1.set_condition(duel, Gemini::effect_status_condition);
    c.register_effect(duel, e1);
}
