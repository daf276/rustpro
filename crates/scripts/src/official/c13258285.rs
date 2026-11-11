// Ukiyoe-P.U.N.K. Sharakusai
use crate::prelude::*;
use crate::testing::{Check, EXTRA, MZONE};

const ID: u32 = 13258285;

struct Params;

impl FusionSummonParams for Params {
    fn fusfilter(c: CardId, duel: &Duel) -> bool {
        c.is_set_card(duel, &[SET_PUNK])
    }
}

// On the field with Ze Amin (a Tuner) and Shapesnatch, the P.U.N.K. materials and the Level 3 + 5 of Dragon Drive, with Rising Carp and Dragon Drive in the Extra Deck.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (19535693, MZONE),
    (19535693, MZONE),
    (4035199, MZONE),
    (4035199, MZONE),
    (18313046, EXTRA),
    (28403802, EXTRA),
    (28403802, EXTRA),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON + CATEGORY_FUSION_SUMMON);
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_count_limit(duel, 1, ID);
    e1.set_cost(duel, Cost::pay_lp::<600>);
    e1.set_target(duel, Fusion::summon_eff_tg::<Params>);
    e1.set_operation(duel, Fusion::summon_eff_op::<Params>);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e2.set_type(duel, EFFECT_TYPE_QUICK_O);
    e2.set_code(duel, EVENT_FREE_CHAIN);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_hint_timing(duel, 0, TIMINGS_CHECK_MONSTER_E | TIMING_MAIN_END);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_condition(duel, |duel, a| duel.is_turn_player(1 - a.tp));
    e2.set_cost(duel, Cost::pay_lp::<600>);
    e2.set_target(duel, sctg);
    e2.set_operation(duel, scop);
    c.register_effect(duel, e2);
}

fn scfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_PUNK]) && c.is_synchro_summonable(duel, None, None)
}

fn sctg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    stock::exists_target(duel, a, chk, scfilter, LOCATION_EXTRA, CATEGORY_SPECIAL_SUMMON)
}

fn scop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    let g = duel.get_matching_group(scfilter, tp, LOCATION_EXTRA, 0, None);
    if !g.is_empty() {
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
        if let Some(sc) = g.select(duel, tp, 1, 1, None)?.get_first() {
            duel.synchro_summon(tp, sc, None, None)?;
        }
    }
    Ok(())
}
