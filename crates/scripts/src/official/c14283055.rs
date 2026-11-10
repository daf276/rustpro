// Concours de Cuisine (Culinary Confrontation)
use crate::prelude::*;
use crate::testing::{Check, DECK, EXTRA, HAND};

const ID: u32 = 14283055;
const LOCATIONS: u32 = LOCATION_HAND | LOCATION_DECK | LOCATION_EXTRA;

// In the hand, with a Nouvelles Pendulum Monster in the Deck and a Patissciel one in the Extra Deck.
pub const SUPPORT: &[(u32, u16)] = &[(ID, HAND), (46485778, DECK), (46485778, DECK), (26435595, EXTRA), (26435595, EXTRA)];
// No Recipe card is ported, which its second effect counts in the Graveyards.
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::SpecialSummonedCode(26435595),
    Check::Report(&Check::ResolvesEffect(aux::stringid(ID, 1))),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_hint_timing(duel, 0, TIMING_MAIN_END | TIMINGS_CHECK_MONSTER);
    e1.set_count_limit(duel, 1, ID);
    e1.set_target(duel, target);
    e1.set_operation(duel, operation);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_ATKCHANGE);
    e2.set_type(duel, EFFECT_TYPE_IGNITION);
    e2.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e2.set_range(duel, LOCATION_GRAVE);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_cost(duel, Cost::self_banish);
    e2.set_target(duel, atktg);
    e2.set_operation(duel, atkop);
    c.register_effect(duel, e2);
}

fn cfilter(c: CardId, duel: &Duel, archetype: u16) -> bool {
    c.is_set_card(duel, &[archetype]) && c.is_type(duel, TYPE_PENDULUM)
}

fn firstsummon(c: CardId, duel: &Duel, e: EffectId, tp: u8, sg: CardSet) -> bool {
    c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
        && ((!c.is_location(duel, LOCATION_EXTRA) && duel.get_location_count(tp, LOCATION_MZONE) > 0)
            || duel.get_location_count_from_ex(tp, tp, None, c) > 0)
        && sg.is_exists(duel, |sc, duel| secondsummon(sc, duel, e, tp), 1, Some(c.into()))
}

fn secondsummon(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_can_be_special_summoned_full(duel, e, 0, tp, false, false, POS_FACEUP, 1 - tp, 0xff)
        && ((!c.is_location(duel, LOCATION_EXTRA)
            && duel.get_location_count_full(1 - tp, LOCATION_MZONE, tp, LOCATION_REASON_TOFIELD, 0xff) > 0)
            || duel.get_location_count_from_ex(1 - tp, tp, None, c) > 0)
}

fn rescon(duel: &Duel, sg: CardSet, e: EffectId, tp: u8) -> (bool, bool) {
    let is_set = |setcode| sg.filter_count(duel, |c, duel| c.is_set_card(duel, &[setcode]), None) == 1;
    (
        is_set(SET_PATISSCIEL) && is_set(SET_NOUVELLES) && sg.is_exists(duel, |c, duel| firstsummon(c, duel, e, tp, sg), 1, None),
        false,
    )
}

fn target(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let g1 = duel.get_matching_group(|c, duel| cfilter(c, duel, SET_NOUVELLES), tp, LOCATIONS, 0, None);
    let g2 = duel.get_matching_group(|c, duel| cfilter(c, duel, SET_PATISSCIEL), tp, LOCATIONS, 0, None);
    let g = g1.union(g2);
    if chk == 0 {
        return Ok(!g1.is_empty()
            && !g2.is_empty()
            && duel.is_player_affected_by_effect(tp, CARD_BLUEEYES_SPIRIT).is_none()
            && aux::select_unselect_group_check(duel, g, 2, 2, &|duel, sg, _, _| rescon(duel, sg, e, tp)));
    }
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 2, tp, LOCATIONS as i32);
    Ok(false)
}

fn operation(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    if duel.is_player_affected_by_effect(tp, CARD_BLUEEYES_SPIRIT).is_some() {
        return Ok(());
    }
    let c = e.get_handler(duel);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_property(duel, EFFECT_FLAG_SET_AVAILABLE + EFFECT_FLAG_IGNORE_IMMUNE);
    e1.set_code(duel, EFFECT_CANNOT_BE_MATERIAL);
    e1.set_target_range(duel, LOCATION_ALL, LOCATION_ALL);
    e1.set_target_card(duel, |duel, _, c| !c.is_set_card(duel, &[SET_NOUVELLES, SET_PATISSCIEL]));
    e1.set_value(duel, Value::Material(sumlimit));
    e1.set_reset(duel, RESET_PHASE | PHASE_END);
    duel.register_effect(e1, tp);
    aux::register_client_hint(duel, c, 0, tp, 1, 0, Some(aux::stringid(ID, 2)));
    let g1 = duel.get_matching_group(|c, duel| cfilter(c, duel, SET_NOUVELLES), tp, LOCATIONS, 0, None);
    let g2 = duel.get_matching_group(|c, duel| cfilter(c, duel, SET_PATISSCIEL), tp, LOCATIONS, 0, None);
    if g1.is_empty() || g2.is_empty() {
        return Ok(());
    }
    let mut sg = aux::select_unselect_group(
        duel,
        g1.union(g2),
        2,
        2,
        &|duel, sg, _, _| rescon(duel, sg, e, tp),
        tp,
        HINTMSG_SPSUMMON,
    )?;
    if sg.len() != 2 {
        return Ok(());
    }
    duel.hint(HINT_SELECTMSG, tp, aux::stringid(ID, 3));
    let first = sg
        .filter_select(duel, tp, |c, duel| firstsummon(c, duel, e, tp, sg), 1, 1, None)?
        .get_first();
    let Some(sc1) = first else { return Ok(()) };
    sg.remove(sc1);
    let Some(sc2) = sg.get_first() else { return Ok(()) };
    duel.special_summon_step(sc1, 0, tp, tp, false, false, POS_FACEUP)?;
    duel.special_summon_step(sc2, 0, tp, 1 - tp, false, false, POS_FACEUP)?;
    duel.special_summon_complete()?;
    Ok(())
}

fn sumlimit(duel: &Duel, e: EffectId, sumc: Option<CardId>, sumtype: u64, sumplayer: u8) -> bool {
    let Some(sumc) = sumc else { return false };
    let tp = e.get_handler_player(duel);
    let allowed = [SUMMON_TYPE_FUSION, SUMMON_TYPE_SYNCHRO, SUMMON_TYPE_XYZ, SUMMON_TYPE_LINK];
    sumplayer == tp && sumc.is_controler(duel, tp) && aux::cannotmatfilter(&allowed.map(u64::from), sumtype)
}

fn matcount(duel: &Duel, tp: u8) -> usize {
    duel.get_matching_group_count(
        |c, duel| c.is_set_card(duel, &[SET_RECIPE]),
        tp,
        LOCATION_GRAVE,
        LOCATION_GRAVE,
        None,
    )
}

fn atktg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    let ct = matcount(duel, a.tp);
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_MZONE) && chkc.is_faceup(duel));
    }
    if chk == 0 {
        return Ok(ct > 0 && duel.is_existing_target(CardId::is_faceup, tp, LOCATION_MZONE, LOCATION_MZONE, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_FACEUP);
    duel.select_target(tp, CardId::is_faceup, tp, LOCATION_MZONE, LOCATION_MZONE, 1, 1, None)?;
    Ok(false)
}

fn atkop(duel: &mut Duel, a: &Args) -> Script<()> {
    let ct = matcount(duel, a.tp);
    if ct == 0 {
        return Ok(());
    }
    if let Some(tc) = duel.get_first_target()
        && tc.is_relate_to_effect(duel, a.e)
        && tc.is_faceup(duel)
    {
        tc.update_attack(
            duel,
            ct as i32 * 200,
            Some(RESET_EVENT | RESETS_STANDARD),
            Some(a.e.get_handler(duel)),
            None,
        );
    }
    Ok(())
}
