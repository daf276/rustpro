// The Phantom Knights' Rank-Up-Magic Requiem
use crate::prelude::*;
use crate::testing::{Check, EXTRA, GRAVE, HAND, MZONE};

const ID: u32 = 62104532;

pub const LISTED_SERIES: &[u16] = &[SET_THE_PHANTOM_KNIGHTS, SET_XYZ_DRAGON];
// Doomed Soleret (to revive), Break Sword (the DARK Xyz Monster) and Raider's Knight (the Rank 4 "The Phantom Knights" Xyz
// Monster). No ported Xyz Monster it can summon has a rum_limit, so that check is left out.
pub const SUPPORT: &[(u32, u16)] = &[(ID, HAND), (83566725, GRAVE), (62709239, MZONE), (28781003, EXTRA)];
pub const CHECKS: &[Check] = &[Check::Resolves, Check::SpecialSummonedCode(28781003)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON + CATEGORY_DISABLE);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_count_limit_full(duel, 1, ID, EFFECT_COUNT_CODE_OATH);
    e1.set_target(duel, target);
    e1.set_operation(duel, activate);
    c.register_effect(duel, e1);
}

fn fieldxyzfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    let pg = aux::get_must_be_material_group(duel, tp, Some(c.into()), tp, None, None, REASON_XYZ);
    if !(pg.is_empty() || (pg.len() == 1 && pg.contains(c))) {
        return false;
    }
    c.is_attribute(duel, ATTRIBUTE_DARK)
        && c.is_xyz_monster(duel)
        && c.is_faceup(duel)
        && duel.is_existing_matching_card(
            |sc, duel| xyzspfilter(sc, duel, e, tp, c, c.get_rank(duel) + 1),
            tp,
            LOCATION_EXTRA,
            0,
            1,
            None,
        )
}

fn xyzspfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8, mc: CardId, rank: u32) -> bool {
    c.is_set_card(duel, &[SET_THE_PHANTOM_KNIGHTS, SET_XYZ_DRAGON])
        && c.is_xyz_monster(duel)
        && c.is_rank(duel, &[rank])
        && mc.is_can_be_xyz_material(duel, Some(c), tp, REASON_XYZ | REASON_MATERIAL)
        && mc.is_type_full(duel, TYPE_XYZ, Some(c), SUMMON_TYPE_XYZ.into(), tp)
        && duel.get_location_count_from_ex(tp, tp, Some(mc.into()), c) > 0
        && c.is_can_be_special_summoned(duel, e, SUMMON_TYPE_XYZ, tp, false, false)
}

fn gybanspfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8, field_xyz_chk: bool) -> bool {
    c.is_set_card(duel, &[SET_THE_PHANTOM_KNIGHTS, SET_XYZ_DRAGON])
        && c.is_faceup(duel)
        && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
        && (field_xyz_chk || fieldxyzfilter(c, duel, e, tp))
}

fn target(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let field_xyz_chk = duel.is_existing_matching_card(|c, duel| fieldxyzfilter(c, duel, e, tp), tp, LOCATION_MZONE, 0, 1, None);
    let filter = |c: CardId, duel: &Duel| gybanspfilter(c, duel, e, tp, field_xyz_chk);
    if let Some(chkc) = chkc {
        return Ok(chkc.is_controler(duel, tp) && chkc.is_location(duel, LOCATION_GRAVE | LOCATION_REMOVED) && filter(chkc, duel));
    }
    if chk == 0 {
        return Ok(duel.is_player_can_special_summon_count(tp, 2)
            && duel.get_location_count(tp, LOCATION_MZONE) > 0
            && duel.is_existing_target(filter, tp, LOCATION_GRAVE | LOCATION_REMOVED, 0, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
    let g = duel.select_target(tp, filter, tp, LOCATION_GRAVE | LOCATION_REMOVED, 0, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, g, 2, tp, LOCATION_EXTRA as i32);
    Ok(false)
}

fn activate(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let Some(tc) = duel.get_first_target() else { return Ok(()) };
    if tc.is_relate_to_effect(duel, e) && duel.special_summon_step(tc, 0, tp, tp, false, false, POS_FACEUP)? {
        tc.negate_effects(duel, e.get_handler(duel), 0, false);
        if duel.special_summon_complete()? == 0 {
            return Ok(());
        }
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_XMATERIAL);
        let Some(mc) = duel
            .select_matching_card(tp, |c, duel| fieldxyzfilter(c, duel, e, tp), tp, LOCATION_MZONE, 0, 1, 1, None)?
            .get_first()
        else {
            return Ok(());
        };
        duel.hint_selection(mc);
        if mc.is_immune_to_effect(duel, e) {
            return Ok(());
        }
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
        let rank = mc.get_rank(duel) + 1;
        let g = duel.select_matching_card(
            tp,
            |c, duel| xyzspfilter(c, duel, e, tp, mc, rank),
            tp,
            LOCATION_EXTRA,
            0,
            1,
            1,
            None,
        )?;
        if let Some(xyz) = g.get_first() {
            xyz.set_material(duel, Some(mc.into()));
            duel.overlay(xyz, mc)?;
            duel.break_effect()?;
            if duel.special_summon(xyz, SUMMON_TYPE_XYZ, tp, tp, false, false, POS_FACEUP)? > 0 {
                xyz.complete_procedure(duel);
            }
        }
    }
    Ok(())
}
