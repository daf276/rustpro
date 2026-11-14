// The Phantom Knights' Rank-Up-Magic Launch
use crate::prelude::*;
use crate::testing::{Check, EXTRA, GRAVE, HAND, MZONE};

const ID: u32 = 3298689;

pub const LISTED_SERIES: &[u16] = &[SET_THE_PHANTOM_KNIGHTS];
// Break Sword (the DARK Xyz Monster) on the field and Raider's Knight (the Rank 4 DARK Xyz Monster) in the Extra Deck; a second
// copy in the Graveyard and Doomed Soleret in the hand to attach. No ported Xyz Monster has a rum_limit, so that check is left out.
pub const SUPPORT: &[(u32, u16)] = &[(ID, HAND), (ID, GRAVE), (83566725, HAND), (62709239, MZONE), (28781003, EXTRA)];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(0),
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::SpecialSummonedCode(28781003),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_hint_timing(duel, 0, TIMING_MAIN_END);
    e1.set_condition(duel, |duel, _| duel.is_main_phase());
    e1.set_target(duel, target);
    e1.set_operation(duel, activate);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 0));
    e2.set_type(duel, EFFECT_TYPE_IGNITION);
    e2.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e2.set_range(duel, LOCATION_GRAVE);
    e2.set_cost(duel, Cost::self_banish);
    e2.set_target(duel, mattg);
    e2.set_operation(duel, matop);
    c.register_effect(duel, e2);
}

fn filter1(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    let rk = c.get_rank(duel);
    let pg = aux::get_must_be_material_group(duel, tp, Some(c.into()), tp, None, None, REASON_XYZ);
    pg.len() <= 1
        && c.is_faceup(duel)
        && c.get_overlay_count(duel) == 0
        && (rk > 0 || c.is_status(duel, STATUS_NO_LEVEL))
        && duel.is_existing_matching_card(|sc, duel| filter2(sc, duel, e, tp, c, rk + 1, pg), tp, LOCATION_EXTRA, 0, 1, None)
}

fn filter2(c: CardId, duel: &Duel, e: EffectId, tp: u8, mc: CardId, rk: u32, pg: CardSet) -> bool {
    duel.get_location_count_from_ex(tp, tp, Some(mc.into()), c) > 0
        && c.is_type(duel, TYPE_XYZ)
        && mc.is_type_full(duel, TYPE_XYZ, Some(c), SUMMON_TYPE_XYZ.into(), tp)
        && mc.is_attribute_full(duel, ATTRIBUTE_DARK, Some(c), SUMMON_TYPE_XYZ.into(), tp)
        && c.is_rank(duel, &[rk])
        && c.is_attribute(duel, ATTRIBUTE_DARK)
        && mc.is_can_be_xyz_material(duel, Some(c), tp, REASON_XYZ | REASON_MATERIAL)
        && (pg.is_empty() || pg.contains(mc))
        && c.is_can_be_special_summoned(duel, e, SUMMON_TYPE_XYZ, tp, false, false)
}

fn target(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let filter = |c, duel: &Duel| filter1(c, duel, e, tp);
    if let Some(chkc) = chkc {
        return Ok(chkc.is_controler(duel, tp) && chkc.is_location(duel, LOCATION_MZONE) && filter(chkc, duel));
    }
    if chk == 0 {
        return Ok(e.is_has_type(duel, EFFECT_TYPE_ACTIVATE) && duel.is_existing_target(filter, tp, LOCATION_MZONE, 0, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TARGET);
    duel.select_target(tp, filter, tp, LOCATION_MZONE, 0, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, tp, LOCATION_EXTRA as i32);
    Ok(false)
}

fn activate(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    let Some(tc) = duel.get_first_target() else { return Ok(()) };
    if tc.is_facedown(duel) || !tc.is_relate_to_effect(duel, e) || tc.is_controler(duel, 1 - tp) || tc.is_immune_to_effect(duel, e) {
        return Ok(());
    }
    let pg = aux::get_must_be_material_group(duel, tp, Some(tc.into()), tp, None, None, REASON_XYZ);
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
    let rk = tc.get_rank(duel) + 1;
    let g = duel.select_matching_card(
        tp,
        |sc, duel| filter2(sc, duel, e, tp, tc, rk, pg),
        tp,
        LOCATION_EXTRA,
        0,
        1,
        1,
        None,
    )?;
    if let Some(sc) = g.get_first() {
        sc.set_material(duel, Some(tc.into()));
        duel.overlay(sc, tc)?;
        duel.special_summon(sc, SUMMON_TYPE_XYZ, tp, tp, false, false, POS_FACEUP)?;
        sc.complete_procedure(duel);
        if c.is_relate_to_effect(duel, e) {
            c.cancel_to_grave(duel);
            duel.overlay(sc, c)?;
        }
    }
    Ok(())
}

fn xyzfilter(c: CardId, duel: &Duel) -> bool {
    c.is_faceup(duel) && c.is_attribute(duel, ATTRIBUTE_DARK) && c.is_type(duel, TYPE_XYZ)
}

fn matfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_THE_PHANTOM_KNIGHTS]) && c.is_monster(duel)
}

fn mattg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_controler(duel, tp) && chkc.is_location(duel, LOCATION_MZONE) && xyzfilter(chkc, duel));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(xyzfilter, tp, LOCATION_MZONE, 0, 1, None)
            && duel.is_existing_matching_card(matfilter, tp, LOCATION_HAND, 0, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_FACEUP);
    duel.select_target(tp, xyzfilter, tp, LOCATION_MZONE, 0, 1, 1, None)?;
    Ok(false)
}

fn matop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    if let Some(tc) = duel.get_first_target()
        && tc.is_faceup(duel)
        && tc.is_relate_to_effect(duel, e)
        && !tc.is_immune_to_effect(duel, e)
    {
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_FACEUP);
        let g = duel.select_matching_card(tp, matfilter, tp, LOCATION_HAND, 0, 1, 1, None)?;
        if !g.is_empty() {
            duel.overlay(tc, g)?;
        }
    }
    Ok(())
}
