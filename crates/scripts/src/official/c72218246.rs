// Crossrose Dragon
use crate::prelude::*;
use crate::testing::{Check, EXTRA, GRAVE, HAND, MZONE, OPPONENT};

const ID: u32 = 72218246;

struct Procedure;

impl LinkProcedure for Procedure {
    const MIN: usize = 2;

    fn specialchk(duel: &Duel, sg: CardSet, lc: CardId, sumtype: u64, tp: u8) -> bool {
        aux::check_different_property_binary(duel, sg, &|c, duel| c.get_race_full(duel, Some(lc), sumtype, tp))
    }
}

// Link Summoned from a Rose Girl, Red Rose Dragon and Plants, with Rose Plant Synchros in the Extra Deck, and copies and Rose Dragons
// in the Graveyard with Raigeki from the opponent to destroy a monster.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (53325667, EXTRA),
    (4290468, EXTRA),
    (29177818, MZONE),
    (26118970, MZONE),
    (14181608, MZONE),
    (14181608, MZONE),
    (ID, GRAVE),
    (26118970, GRAVE),
    (93708824, GRAVE),
    (12580477, HAND | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Link::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_type(duel, EFFECT_TYPE_QUICK_O);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_count_limit(duel, 1, ID);
    e1.set_hint_timing(duel, 0, TIMING_MAIN_END);
    e1.set_condition(duel, |duel, _| duel.is_main_phase());
    e1.set_cost(duel, spcost);
    e1.set_target(duel, stock::operation_info_target::<CATEGORY_SPECIAL_SUMMON, LOCATION_EXTRA>);
    e1.set_operation(duel, spop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e2.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e2.set_property(duel, EFFECT_FLAG_DELAY);
    e2.set_code(duel, EVENT_DESTROYED);
    e2.set_range(duel, LOCATION_GRAVE);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_condition(duel, spcon2);
    e2.set_cost(duel, Cost::self_banish);
    e2.set_target(duel, sptg2);
    e2.set_operation(duel, spop2);
    c.register_effect(duel, e2);
}

fn mzfilter(c: CardId, duel: &Duel) -> bool {
    c.is_race(duel, RACE_PLANT)
}

fn exkfilter(c: CardId, duel: &Duel, sg: CardSet, tp: u8, oc: CardId) -> bool {
    duel.get_location_count_from_ex(tp, tp, Some(sg.union(oc.into())), c) > 0
}

fn excheck(duel: &Duel, sg: CardSet, tp: u8, ssg: CardSet, c: CardId) -> bool {
    ssg.is_exists(duel, |sc, duel| exkfilter(sc, duel, sg, tp, c), 1, None)
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8, chk: bool) -> bool {
    (c.is_set_card(duel, &[SET_ROSE]) || c.is_race(duel, RACE_PLANT))
        && c.is_type(duel, TYPE_SYNCHRO)
        && (!chk || duel.get_location_count_from_ex(tp, tp, None, c) > 0)
        && c.is_can_be_special_summoned(duel, e, SUMMON_TYPE_SYNCHRO, tp, false, false)
}

fn spcost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    let mg = duel.get_matching_group(|sc, duel| spfilter(sc, duel, e, tp, false), tp, LOCATION_EXTRA, 0, None);
    let check = |duel: &Duel, sg: CardSet, _: u8, _: CardSet| (excheck(duel, sg, tp, mg, c), false);
    if chk == 0 {
        return Ok(duel.check_release_group_cost(tp, mzfilter, (1, 1), false, Some(&check), Some(c.into())));
    }
    let mut g = duel.select_release_group_cost(tp, mzfilter, (1, 1), false, Some(&check), Some(c.into()))?;
    g.insert(c);
    duel.release(g, REASON_COST)?;
    Ok(false)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
    let g = duel.select_matching_card(tp, |c, duel| spfilter(c, duel, e, tp, true), tp, LOCATION_EXTRA, 0, 1, 1, None)?;
    if let Some(tc) = g.get_first()
        && duel.special_summon(tc, SUMMON_TYPE_SYNCHRO, tp, tp, false, false, POS_FACEUP)? > 0
    {
        tc.complete_procedure(duel);
    }
    Ok(())
}

fn cfilter(c: CardId, duel: &Duel, tp: u8) -> bool {
    c.get_previous_controler(duel) == tp && c.is_reason(duel, REASON_EFFECT) && c.get_previous_type_on_field(duel) & TYPE_MONSTER != 0
}

fn spcon2(duel: &Duel, a: &Args) -> bool {
    let (c, tp) = (a.e.get_handler(duel), a.tp);
    a.eg.is_some_and(|eg| !eg.is_contains(c) && eg.is_exists(duel, |sc, duel| cfilter(sc, duel, tp), 1, None))
}

fn spfilter2(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_set_card(duel, &[SET_ROSE_DRAGON]) && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
}

fn sptg2(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if chk == 0 {
        let c = e.get_handler(duel);
        return Ok(duel.get_location_count(tp, LOCATION_MZONE) > 0
            && duel.is_existing_matching_card(|sc, duel| spfilter2(sc, duel, e, tp), tp, LOCATION_GRAVE, 0, 1, Some(c.into())));
    }
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, tp, LOCATION_GRAVE as i32);
    Ok(false)
}

fn spop2(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    if duel.get_location_count(tp, LOCATION_MZONE) <= 0 {
        return Ok(());
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
    let filter = |c, duel: &Duel| spfilter2(c, duel, e, tp);
    let g = duel.select_matching_card(tp, aux::necro_valley_filter(filter), tp, LOCATION_GRAVE, 0, 1, 1, None)?;
    if !g.is_empty() {
        duel.special_summon(g, 0, tp, tp, false, false, POS_FACEUP)?;
    }
    Ok(())
}
