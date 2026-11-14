// Darktellarknight Batlamyus
use crate::prelude::*;
use crate::testing::{Check, EXTRA, MZONE};

const ID: u32 = 64414267;

pub const LISTED_SERIES: &[u16] = &[SET_TELLARKNIGHT];
// Xyz Summoned from the Extra Deck with two Level 4 "tellarknight" monsters (Satellarknight Deneb, Unukalhai), with Tellarknight
// Ptolemaeus in the Extra Deck to Xyz Summon. The Quick Effect needs seven different "tellarknight" monsters in the Graveyard, which
// only three ported cards can be, so only the Ignition Effect resolves.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (75878039, MZONE),
    (75878039, MZONE),
    (1050186, MZONE),
    (1050186, MZONE),
    (18326736, EXTRA),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0))];

struct Procedure;

impl XyzProcedure for Procedure {
    const F: Option<XyzFilter> =
        Some(|c, duel, scard, sumtype, tp| c.is_set_card_full(duel, &[SET_TELLARKNIGHT], Some(scard), sumtype, tp));
    const LV: Option<u32> = Some(4);
    const CT: usize = 2;
}

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Xyz::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_code(duel, EFFECT_CHANGE_ATTRIBUTE);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_target_range(duel, LOCATION_MZONE, LOCATION_MZONE);
    e1.set_value(duel, ATTRIBUTE_DARK as i32);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 0));
    e2.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e2.set_type(duel, EFFECT_TYPE_IGNITION);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_count_limit_full(duel, 1, 0, EFFECT_COUNT_CODE_CHAIN);
    e2.set_condition(duel, |duel, a| !quickcon(duel, a));
    e2.set_cost(duel, |duel, a, chk| {
        Cost::and(duel, a, chk, &[Cost::detach_from_self::<1, 1>, Cost::discard])
    });
    e2.set_target(duel, sptg);
    e2.set_operation(duel, spop);
    c.register_effect(duel, e2);
    let e3 = e2.clone_effect(duel);
    e3.set_type(duel, EFFECT_TYPE_QUICK_O);
    e3.set_code(duel, EVENT_FREE_CHAIN);
    e3.set_hint_timing(duel, 0, TIMING_STANDBY_PHASE | TIMING_MAIN_END | TIMINGS_CHECK_MONSTER_E);
    e3.set_condition(duel, quickcon);
    c.register_effect(duel, e3);
}

fn quickconfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_TELLARKNIGHT]) && c.is_monster(duel)
}

fn quickcon(duel: &Duel, a: &Args) -> bool {
    duel.get_matching_group(quickconfilter, a.tp, LOCATION_GRAVE, 0, None)
        .get_class_count(duel, |c, duel| i64::from(c.get_code(duel)))
        >= 7
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8, mc: CardId) -> bool {
    c.is_attribute(duel, ATTRIBUTE_LIGHT)
        && c.is_set_card(duel, &[SET_TELLARKNIGHT])
        && c.is_type(duel, TYPE_XYZ)
        && duel.get_location_count_from_ex(tp, tp, Some(mc.into()), c) > 0
        && mc.is_can_be_xyz_material(duel, Some(c), tp, REASON_XYZ | REASON_MATERIAL)
        && c.is_can_be_special_summoned(duel, e, SUMMON_TYPE_XYZ, tp, false, false)
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if chk == 0 {
        let c = e.get_handler(duel);
        let pg = aux::get_must_be_material_group(duel, tp, Some(c.into()), tp, None, None, REASON_XYZ);
        return Ok((pg.is_empty() || (pg.len() == 1 && pg.contains(c)))
            && duel.is_existing_matching_card(|sc, duel| spfilter(sc, duel, e, tp, c), tp, LOCATION_EXTRA, 0, 1, None));
    }
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, tp, LOCATION_EXTRA as i32);
    Ok(false)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    let pg = aux::get_must_be_material_group(duel, tp, Some(c.into()), tp, None, None, REASON_XYZ);
    if c.is_faceup(duel)
        && c.is_relate_to_effect(duel, e)
        && c.is_controler(duel, tp)
        && !c.is_immune_to_effect(duel, e)
        && (pg.is_empty() || (pg.len() == 1 && pg.contains(c)))
    {
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
        let g = duel.select_matching_card(tp, |sc, duel| spfilter(sc, duel, e, tp, c), tp, LOCATION_EXTRA, 0, 1, 1, None)?;
        if let Some(xyzc) = g.get_first() {
            xyzc.set_material(duel, Some(c.into()));
            duel.overlay(xyzc, c)?;
            if duel.special_summon(xyzc, SUMMON_TYPE_XYZ, tp, tp, false, false, POS_FACEUP)? > 0 {
                xyzc.complete_procedure(duel);
            }
        }
    }
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 1));
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_property(duel, EFFECT_FLAG_PLAYER_TARGET + EFFECT_FLAG_CLIENT_HINT);
    e1.set_code(duel, EFFECT_CANNOT_SPECIAL_SUMMON);
    e1.set_target_range(duel, 1, 0);
    e1.set_target_spsummon(duel, |_, _, _, _, sumtype, _, _, _| sumtype & SUMMON_TYPE_XYZ == SUMMON_TYPE_XYZ);
    e1.set_reset(duel, RESET_PHASE | PHASE_END);
    duel.register_effect(e1, tp);
    Ok(())
}
