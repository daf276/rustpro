// Spirit Message "I"
use crate::prelude::*;
use crate::testing::Check;

// Only restricts being Set and Special Summoned, so no effect resolves.
pub const CHECKS: &[Check] = &[Check::Report(&Check::Resolves)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_code(duel, EFFECT_CANNOT_SSET);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_SINGLE);
    e2.set_code(duel, EFFECT_SPSUMMON_COST);
    e2.set_cost_condition(duel, |_, _, _, _, sumtype| sumtype == SUMMON_TYPE_SPECIAL + 181);
    c.register_effect(duel, e2);
}
