// Asura Priest
use crate::prelude::*;
use crate::testing::HAND;

// In the hand, to be Normal Summoned and return in the End Phase.
pub const SUPPORT: &[(u32, u16)] = &[(2134346, HAND)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    Spirit::add_procedure(duel, c, &[EVENT_SUMMON_SUCCESS, EVENT_FLIP]);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE + EFFECT_FLAG_UNCOPYABLE);
    e1.set_code(duel, EFFECT_SPSUMMON_CONDITION);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_SINGLE);
    e2.set_code(duel, EFFECT_ATTACK_ALL);
    e2.set_value(duel, 1);
    c.register_effect(duel, e2);
}
