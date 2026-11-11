// Blackwing - Kochi the Daybreak
use crate::prelude::*;
use crate::testing::Check;

// Only restricts its own use as a Synchro Material and Special Summon, so no effect resolves.
pub const CHECKS: &[Check] = &[Check::Report(&Check::Resolves)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE + EFFECT_FLAG_UNCOPYABLE);
    e1.set_code(duel, EFFECT_CANNOT_BE_SYNCHRO_MATERIAL);
    e1.set_condition(duel, synlimit);
    e1.set_value(duel, 1);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_SINGLE);
    e2.set_code(duel, EFFECT_SPSUMMON_COST);
    e2.set_cost_condition(duel, |_, _, _, _, sumtype| sumtype != SUMMON_TYPE_SPECIAL + 182);
    c.register_effect(duel, e2);
}

fn synlimit(duel: &Duel, a: &Args) -> bool {
    a.e.get_handler(duel).is_special_summoned(duel)
}
