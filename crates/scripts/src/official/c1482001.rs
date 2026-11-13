// Clara & Rushka, the Ventriloduo
use crate::prelude::*;
use crate::testing::{Check, EXTRA, HAND};

const ID: u32 = 1482001;

struct Procedure;

impl LinkProcedure for Procedure {
    const MIN: usize = 1;
    const MAX: Option<usize> = Some(1);

    fn f(c: CardId, duel: &Duel, _: CardId, _: u64, _: u8) -> bool {
        c.is_summon_type(duel, &[SUMMON_TYPE_NORMAL])
    }

    fn spcon(duel: &Duel, _: EffectId, _: u8) -> bool {
        splimit(duel, SUMMON_TYPE_LINK)
    }
}

// Copies in the Extra Deck to Link Summon in the Main Phase 2 with a Normal Summoned monster, and Normal Monsters in the hand.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (ID, EXTRA),
    (32295838, HAND),
    (36211150, HAND),
    (32295838, HAND),
    (36211150, HAND),
];
pub const CHECKS: &[Check] = &[Check::SpecialSummoned];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Link::add_procedure::<Procedure>(duel, c);
    let e0 = duel.create_effect(c);
    e0.set_type(duel, EFFECT_TYPE_SINGLE);
    e0.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE + EFFECT_FLAG_UNCOPYABLE);
    e0.set_code(duel, EFFECT_SPSUMMON_CONDITION);
    e0.set_value(duel, Value::SpSummonCondition(|duel, _, _, _, st, _, _| splimit(duel, st)));
    c.register_effect(duel, e0);
}

fn splimit(duel: &Duel, st: u32) -> bool {
    st & SUMMON_TYPE_LINK != SUMMON_TYPE_LINK || duel.is_phase(PHASE_MAIN2)
}
