// Gamma Summon
use crate::prelude::*;
use crate::testing::{Check, EXTRA, GRAVE, HAND, MZONE, REMOVED};

const ID: u32 = 86319972;

// Aleister the Reminiscent and Invoked Elysium (Fusion) to banish from the Graveyard, Invoked Sorath and Oceanus in the Extra
// Deck to Special Summon, and a second copy in the Graveyard with a banished Invoked Raidjin to return.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, GRAVE),
    (57294268, GRAVE),
    (11270236, GRAVE),
    (43989315, EXTRA),
    (6772168, EXTRA),
    (49513164, REMOVED),
    (57294268, MZONE),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
    Check::SpecialSummonedCode(43989315),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON + CATEGORY_REMOVE);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_count_limit(duel, 1, (ID, 0));
    e1.set_cost(duel, extraspcost);
    e1.set_target(duel, extrasptg);
    e1.set_operation(duel, extraspop);
    e1.set_hint_timing(duel, 0, TIMING_STANDBY_PHASE | TIMING_MAIN_END | TIMINGS_CHECK_MONSTER_E);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e2.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    // The Lua passes the undefined global EFFECT as the second property argument, so EFFECT_FLAG2_CHECK_SIMULTANEOUS is dropped.
    e2.set_property(duel, EFFECT_FLAG_DELAY + EFFECT_FLAG_CARD_TARGET);
    e2.set_code(duel, EVENT_REMOVE);
    e2.set_range(duel, LOCATION_GRAVE);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_condition(duel, banspcon);
    e2.set_cost(duel, Cost::self_banish);
    e2.set_target(duel, bansptg);
    e2.set_operation(duel, stock::special_summon_first_target::<POS_FACEUP>);
    c.register_effect(duel, e2);
}

fn extraspcostfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    (c.is_set_card(duel, &[SET_ALEISTER]) || c.is_fusion_monster(duel))
        && c.is_able_to_remove_as_cost(duel)
        && c.is_monster(duel)
        && aux::sp_elim_filter(c, duel, true, false)
        && duel.is_existing_matching_card(|sc, duel| extraspfilter(sc, duel, e, tp, Some(c)), tp, LOCATION_EXTRA, 0, 1, None)
}

fn extraspfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8, rc: Option<CardId>) -> bool {
    c.is_set_card(duel, &[SET_INVOKED])
        && duel.get_location_count_from_ex(tp, tp, rc.map(CardSet::from), c) > 0
        && c.is_can_be_special_summoned(duel, e, 0, tp, true, false)
}

fn extraspcost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    e.set_label(duel, -100);
    let locations = LOCATION_MZONE | LOCATION_GRAVE;
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(|c, duel| extraspcostfilter(c, duel, e, tp), tp, locations, 0, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_REMOVE);
    let g = duel.select_matching_card(tp, |c, duel| extraspcostfilter(c, duel, e, tp), tp, locations, 0, 1, 1, None)?;
    duel.remove(g, 0, REASON_COST)?;
    Ok(false)
}

fn extrasptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if chk == 0 {
        let cost_chk = e.get_label(duel) == -100;
        e.set_label(duel, 0);
        return Ok(
            cost_chk || duel.is_existing_matching_card(|c, duel| extraspfilter(c, duel, e, tp, None), tp, LOCATION_EXTRA, 0, 1, None)
        );
    }
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, tp, LOCATION_EXTRA as i32);
    duel.set_possible_operation_info(0, CATEGORY_REMOVE, None, 1, tp, LOCATION_MZONE as i32);
    Ok(false)
}

struct Banish;

impl aux::DelayedOperation for Banish {
    fn operate(duel: &mut Duel, ag: CardSet, _: &Args) -> Script<()> {
        duel.remove(ag, POS_FACEUP, REASON_EFFECT)?;
        Ok(())
    }
}

fn extraspop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
    let g = duel.select_matching_card(tp, |c, duel| extraspfilter(c, duel, e, tp, None), tp, LOCATION_EXTRA, 0, 1, 1, None)?;
    if let Some(sc) = g.get_first()
        && duel.special_summon(sc, 0, tp, tp, true, false, POS_FACEUP)? > 0
    {
        aux::delayed_operation_full::<Banish>(duel, sc.into(), PHASE_END, 0, ID, e, tp, aux::stringid(ID, 2), 0);
    }
    Ok(())
}

fn banspconfilter(c: CardId, duel: &Duel) -> bool {
    c.is_monster(duel) && c.is_faceup(duel) && c.is_location(duel, LOCATION_REMOVED) && !c.is_previous_location(duel, LOCATION_SZONE)
}

fn banspcon(duel: &Duel, a: &Args) -> bool {
    a.eg.unwrap_or_default().is_exists(duel, banspconfilter, 1, None)
}

fn banspfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_set_card(duel, &[SET_INVOKED]) && c.is_faceup(duel) && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
}

fn bansptg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_REMOVED) && chkc.is_controler(duel, tp) && banspfilter(chkc, duel, e, tp));
    }
    if chk == 0 {
        return Ok(duel.get_location_count(tp, LOCATION_MZONE) > 0
            && duel.is_existing_target(|c, duel| banspfilter(c, duel, e, tp), tp, LOCATION_REMOVED, 0, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
    let g = duel.select_target(tp, |c, duel| banspfilter(c, duel, e, tp), tp, LOCATION_REMOVED, 0, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, g, 1, tp, 0);
    Ok(false)
}
