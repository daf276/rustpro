// CXyz N.As.Ch. Knight
use crate::prelude::*;
use crate::testing::{Check, EXTRA, MZONE};

use super::c34876719::is_number_10x;

const ID: u32 = 61374414;

pub const LISTED_SERIES: &[u16] = &[SET_NUMBER];
pub const LISTED_NAMES: &[u32] = &[34876719];

// On the field with Xyz materials, with N.As.H. Knight and Numbers 101, 104, 106 and 107 in the Extra Deck.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (ID, EXTRA),
    (34876719, MZONE),
    (48739166, EXTRA),
    (2061963, EXTRA),
    (63746411, EXTRA),
    (88177324, EXTRA),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 1)), Check::SpecialSummonedCode(48739166)];

struct Procedure;

impl XyzProcedure for Procedure {
    const LV: Option<u32> = Some(6);
    const CT: usize = 3;
    const ALTERF: Option<AlterFilter> = Some(ovfilter);
    const DESC: u64 = aux::stringid(ID, 0);
}

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    Xyz::add_procedure::<Procedure>(duel, c);
    c.enable_revive_limit(duel);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_property(duel, EFFECT_FLAG_SINGLE_RANGE);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_code(duel, EFFECT_INDESTRUCTABLE_EFFECT);
    e1.set_value(duel, 1);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e2.set_type(duel, EFFECT_TYPE_IGNITION);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_count_limit(duel, 1, ID);
    e2.set_cost(duel, Cost::detach_from_self::<1, 1>);
    e2.set_target(duel, sptg);
    e2.set_operation(duel, spop);
    c.register_effect(duel, e2);
}

fn ovfilter(c: CardId, duel: &Duel, tp: u8, lc: CardId) -> bool {
    c.is_faceup(duel) && c.is_summon_code(duel, Some(lc), SUMMON_TYPE_XYZ as u64, tp, &[34876719])
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8, mc: CardId, pg: CardSet) -> bool {
    is_number_10x(c, duel)
        && duel.get_location_count_from_ex(tp, tp, Some(mc.into()), c) > 0
        && (pg.is_empty() || pg.contains(mc))
        && mc.is_can_be_xyz_material(duel, Some(c), tp, REASON_XYZ | REASON_MATERIAL)
        && c.is_can_be_special_summoned(duel, e, SUMMON_TYPE_XYZ, tp, false, false)
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if chk == 0 {
        let c = e.get_handler(duel);
        let pg = aux::get_must_be_material_group(duel, tp, Some(c.into()), tp, None, None, REASON_XYZ);
        return Ok((pg.is_empty() || (pg.len() == 1 && pg.contains(c)))
            && duel.is_existing_matching_card(|sc, duel| spfilter(sc, duel, e, tp, c, pg), tp, LOCATION_EXTRA, 0, 1, None));
    }
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, tp, LOCATION_EXTRA as i32);
    Ok(false)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    let pg = aux::get_must_be_material_group(duel, tp, Some(c.into()), tp, None, None, REASON_XYZ);
    if c.is_faceup(duel) && c.is_relate_to_effect(duel, e) && c.is_controler(duel, tp) && !c.is_immune_to_effect(duel, e) {
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
        let g = duel.select_matching_card(tp, |sc, duel| spfilter(sc, duel, e, tp, c, pg), tp, LOCATION_EXTRA, 0, 1, 1, None)?;
        if let Some(sc) = g.get_first() {
            sc.set_material(duel, Some(c.into()));
            duel.overlay(sc, c)?;
            if duel.special_summon(sc, SUMMON_TYPE_XYZ, tp, tp, false, false, POS_FACEUP)? > 0 {
                sc.complete_procedure(duel);
                let reset = RESETS_STANDARD_PHASE_END | RESET_OPPO_TURN;
                let e1 = duel.create_effect(c);
                e1.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_CONTINUOUS);
                e1.set_code(duel, EVENT_PHASE | PHASE_END);
                e1.set_property(duel, EFFECT_FLAG_IGNORE_IMMUNE);
                e1.set_count_limit(duel, 1, 0);
                e1.set_label_object_group(duel, Some(sc.into()));
                e1.set_condition(duel, descon);
                e1.set_operation(duel, desop);
                e1.set_reset(duel, reset);
                sc.register_flag_effect_full(duel, ID, reset, EFFECT_FLAG_CLIENT_HINT, 1, 0, aux::stringid(ID, 2));
                duel.register_effect(e1, tp);
            }
        }
    }
    Ok(())
}

fn descon(duel: &Duel, a: &Args) -> bool {
    duel.is_turn_player(1 - a.tp)
        && a.e
            .get_label_object_group(duel)
            .and_then(|g| g.get_first())
            .is_some_and(|sc| sc.has_flag_effect(duel, ID))
}

fn desop(duel: &mut Duel, a: &Args) -> Script<()> {
    if let Some(sc) = a.e.get_label_object_group(duel).and_then(|g| g.get_first()) {
        duel.destroy(sc, REASON_EFFECT)?;
    }
    Ok(())
}
