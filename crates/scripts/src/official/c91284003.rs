// Argostars - Swift Capane
use super::c65889305::{trap_sptg, trap_summon};
use crate::prelude::*;
use crate::testing::{Check, HAND, REMOVED, SZONE};

const ID: u32 = 91284003;

pub const LISTED_SERIES: &[u16] = &[SET_ARGOSTARS];
// Set copies and Lightning Tydeus, which put a Continuous Trap in a Monster Zone, and a banished Argostars monster.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, SZONE),
    (ID, SZONE),
    (65889305, SZONE),
    (65889305, SZONE),
    (65889305, HAND),
    (91438674, REMOVED),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(0), Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e0 = duel.create_effect(c);
    e0.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e0.set_code(duel, EVENT_FREE_CHAIN);
    c.register_effect(duel, e0);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_code(duel, EFFECT_INDESTRUCTABLE_BATTLE);
    e1.set_range(duel, LOCATION_SZONE);
    e1.set_target_range(duel, LOCATION_MZONE, 0);
    e1.set_target_card(duel, |duel, _, c| c.is_race(duel, RACE_WARRIOR));
    e1.set_value(duel, 1);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 0));
    e2.set_category(duel, CATEGORY_SPECIAL_SUMMON + CATEGORY_RECOVER);
    e2.set_type(duel, EFFECT_TYPE_QUICK_O);
    e2.set_code(duel, EVENT_FREE_CHAIN);
    e2.set_range(duel, LOCATION_SZONE);
    e2.set_count_limit(duel, 1, 0);
    e2.set_cost(duel, |duel, a, _| {
        let filter = aux::faceup_filter(CardId::is_continuous_trap);
        Ok(duel.is_existing_matching_card(filter, a.tp, LOCATION_MZONE, LOCATION_MZONE, 1, None))
    });
    e2.set_target(duel, sptg);
    e2.set_operation(duel, spop);
    e2.set_hint_timing(duel, 0, TIMING_STANDBY_PHASE | TIMING_MAIN_END | TIMINGS_CHECK_MONSTER_E);
    c.register_effect(duel, e2);
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    trap_sptg::<ID, 1800>(duel, a, chk, (CATEGORY_RECOVER, 0, a.tp, 500))
}

pub fn argosfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_ARGOSTARS]) && c.is_monster(duel) && c.is_faceup(duel)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    if trap_summon::<ID, 1800>(duel, a)?.is_some_and(|count| count > 0)
        && duel.is_existing_matching_card(argosfilter, tp, LOCATION_REMOVED, 0, 1, None)
    {
        duel.break_effect()?;
        duel.recover(tp, 500, REASON_EFFECT)?;
    }
    Ok(())
}
