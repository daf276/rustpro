// Constellar Tellarknights
use crate::prelude::*;
use crate::testing::{Check, EXTRA, GRAVE, HAND, MZONE, SZONE};

const ID: u32 = 10125011;

pub const LISTED_SERIES: &[u16] = &[SET_TELLARKNIGHT, SET_CONSTELLAR];

// Satellarknight Deneb in the hand and Graveyard to Special Summon, and Tellarknight Ptolemaeus (Rank 4) on the field to
// Xyz Summon Constellar Pleiades (Rank 5) with.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, SZONE),
    (75878039, HAND),
    (75878039, GRAVE),
    (18326736, MZONE),
    (18326736, MZONE),
    (73964868, EXTRA),
    (73964868, EXTRA),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_count_limit_full(duel, 1, ID, EFFECT_COUNT_CODE_OATH);
    e1.set_target(duel, target);
    e1.set_operation(duel, activate);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e2.set_type(duel, EFFECT_TYPE_IGNITION);
    e2.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e2.set_range(duel, LOCATION_SZONE);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_target(duel, xyztg);
    e2.set_operation(duel, xyzop);
    c.register_effect(duel, e2);
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_set_card(duel, &[SET_TELLARKNIGHT, SET_CONSTELLAR]) && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
}

fn target(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(true);
    }
    duel.set_possible_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, a.tp, (LOCATION_HAND | LOCATION_GRAVE) as i32);
    Ok(false)
}

fn activate(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    if duel.get_location_count(tp, LOCATION_MZONE) <= 0 {
        return Ok(());
    }
    let g = duel.get_matching_group(
        aux::necro_valley_filter(|c, duel| spfilter(c, duel, e, tp)),
        tp,
        LOCATION_HAND | LOCATION_GRAVE,
        0,
        None,
    );
    if !g.is_empty() && duel.select_yes_no(tp, aux::stringid(ID, 0))? {
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
        let sg = g.select(duel, tp, 1, 1, None)?;
        duel.special_summon(sg, 0, tp, tp, false, false, POS_FACEUP)?;
    }
    Ok(())
}

fn xyzfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8, mc: CardId) -> bool {
    mc.is_type_full(duel, TYPE_XYZ, Some(c), SUMMON_TYPE_XYZ as u64, tp)
        && c.is_type(duel, TYPE_XYZ)
        && c.is_set_card(duel, &[SET_TELLARKNIGHT, SET_CONSTELLAR])
        && duel.get_location_count_from_ex(tp, tp, Some(mc.into()), c) > 0
        && !c.is_rank(duel, &[mc.get_rank(duel)])
        && mc.is_can_be_xyz_material(duel, Some(c), duel.get_reason_player(), REASON_XYZ | REASON_MATERIAL)
        && c.is_can_be_special_summoned(duel, e, SUMMON_TYPE_XYZ, tp, false, false)
}

fn must_be_material_group(duel: &Duel, c: CardId, tp: u8) -> CardSet {
    aux::get_must_be_material_group(duel, tp, Some(c.into()), tp, None, None, REASON_XYZ)
}

fn xyztgfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    if !(c.is_faceup(duel) && c.is_set_card(duel, &[SET_TELLARKNIGHT, SET_CONSTELLAR]) && c.is_type(duel, TYPE_XYZ)) {
        return false;
    }
    let pg = must_be_material_group(duel, c, tp);
    (pg.is_empty() || (pg.len() == 1 && pg.contains(c)))
        && duel.is_existing_matching_card(|sc, duel| xyzfilter(sc, duel, e, tp, c), tp, LOCATION_EXTRA, 0, 1, None)
}

fn xyztg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let filter = |c, duel: &Duel| xyztgfilter(c, duel, e, tp);
    if let Some(chkc) = chkc {
        return Ok(chkc.is_controler(duel, tp) && chkc.is_location(duel, LOCATION_MZONE) && filter(chkc, duel));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(filter, tp, LOCATION_MZONE, 0, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TARGET);
    duel.select_target(tp, filter, tp, LOCATION_MZONE, 0, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, tp, LOCATION_EXTRA as i32);
    Ok(false)
}

fn xyzop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let Some(tc) = duel.get_first_target() else { return Ok(()) };
    if tc.is_facedown(duel) || !tc.is_relate_to_effect(duel, e) || tc.is_controler(duel, 1 - tp) || tc.is_immune_to_effect(duel, e) {
        return Ok(());
    }
    let pg = must_be_material_group(duel, tc, tp);
    if pg.len() > 1 || (pg.len() == 1 && pg.get_first() != Some(tc)) {
        return Ok(());
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
    let g = duel.select_matching_card(tp, |sc, duel| xyzfilter(sc, duel, e, tp, tc), tp, LOCATION_EXTRA, 0, 1, 1, None)?;
    let Some(sc) = g.get_first() else { return Ok(()) };
    sc.set_material(duel, Some(tc.into()));
    duel.overlay(sc, tc)?;
    duel.special_summon(sc, SUMMON_TYPE_XYZ, tp, tp, false, false, POS_FACEUP)?;
    sc.complete_procedure(duel);
    Ok(())
}
