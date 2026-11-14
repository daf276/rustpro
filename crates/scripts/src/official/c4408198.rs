// Exosister Arment
use crate::prelude::*;
use crate::testing::{Check, EXTRA, HAND, MZONE, SZONE};

const ID: u32 = 4408198;

pub const LISTED_SERIES: &[u16] = &[SET_EXOSISTER];

// Set and in the hand, with Exosister Elis and Stella on the field to become the material of Exosister Mikailis or Kaspitell.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, SZONE),
    (ID, SZONE),
    (ID, HAND),
    (16474916, MZONE),
    (43863925, MZONE),
    (42741437, EXTRA),
    (78135071, EXTRA),
];
pub const CHECKS: &[Check] = &[Check::SpecialSummonedCode(42741437), Check::SpecialSummonedCode(78135071)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e1.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_count_limit_full(duel, 1, ID, EFFECT_COUNT_CODE_OATH);
    e1.set_hint_timing(duel, 0, TIMINGS_CHECK_MONSTER_E);
    e1.set_condition(duel, condition);
    e1.set_cost(duel, Cost::pay_lp::<800>);
    e1.set_target(duel, target);
    e1.set_operation(duel, activate);
    c.register_effect(duel, e1);
}

fn condition(duel: &Duel, a: &Args) -> bool {
    duel.is_turn_player(1 - a.tp)
        || duel.is_existing_matching_card(
            |c, duel| c.is_summon_location(duel, LOCATION_GRAVE),
            a.tp,
            0,
            LOCATION_MZONE,
            1,
            None,
        )
}

fn must_materials(duel: &Duel, tp: u8, c: CardId) -> CardSet {
    aux::get_must_be_material_group(duel, tp, Some(c.into()), tp, None, None, REASON_XYZ)
}

fn filter1(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    let pg = must_materials(duel, tp, c);
    (pg.is_empty() || (pg.len() == 1 && pg.contains(c)))
        && c.is_faceup(duel)
        && c.is_set_card(duel, &[SET_EXOSISTER])
        && duel.is_existing_matching_card(|sc, duel| filter2(sc, duel, e, tp, c), tp, LOCATION_EXTRA, 0, 1, None)
}

fn filter2(c: CardId, duel: &Duel, e: EffectId, tp: u8, mc: CardId) -> bool {
    let code = c.get_code(duel);
    c.is_type(duel, TYPE_XYZ)
        && c.is_set_card(duel, &[SET_EXOSISTER])
        && !duel.is_existing_matching_card(
            aux::faceup_filter(|c, duel| c.is_code(duel, code)),
            tp,
            LOCATION_ONFIELD,
            0,
            1,
            None,
        )
        && duel.get_location_count_from_ex(tp, tp, Some(mc.into()), c) > 0
        && mc.is_can_be_xyz_material(duel, Some(c), tp, REASON_XYZ | REASON_MATERIAL)
        && c.is_can_be_special_summoned(duel, e, SUMMON_TYPE_XYZ, tp, false, false)
}

fn target(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if let Some(chkc) = chkc {
        return Ok(chkc.is_controler(duel, tp) && chkc.is_location(duel, LOCATION_MZONE) && filter1(chkc, duel, e, tp));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(|c, duel| filter1(c, duel, e, tp), tp, LOCATION_MZONE, 0, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TARGET);
    duel.select_target(tp, |c, duel| filter1(c, duel, e, tp), tp, LOCATION_MZONE, 0, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, tp, LOCATION_EXTRA as i32);
    Ok(false)
}

fn activate(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let Some(tc) = duel.get_first_target() else { return Ok(()) };
    let pg = must_materials(duel, tp, tc);
    if tc.is_facedown(duel)
        || !tc.is_relate_to_effect(duel, e)
        || tc.is_controler(duel, 1 - tp)
        || tc.is_immune_to_effect(duel, e)
        || pg.len() > 1
        || (pg.len() == 1 && !pg.contains(tc))
    {
        return Ok(());
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
    let g = duel.select_matching_card(tp, |sc, duel| filter2(sc, duel, e, tp, tc), tp, LOCATION_EXTRA, 0, 1, 1, None)?;
    if let Some(sc) = g.get_first() {
        sc.set_material(duel, Some(tc.into()));
        duel.overlay(sc, tc)?;
        duel.special_summon(sc, SUMMON_TYPE_XYZ, tp, tp, false, false, POS_FACEUP)?;
        sc.complete_procedure(duel);
    }
    Ok(())
}
