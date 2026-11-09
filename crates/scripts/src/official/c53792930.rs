// K9-X Forced Release
use crate::prelude::*;
use crate::testing::{Check, EXTRA, GRAVE, HAND, MZONE, OPPONENT};

const ID: u32 = 53792930;

// K9-66X "Jacks" and K9-17 Izuna on the field to battle, with K9-17 "Ripper" and K9-X "Werewolf" in the Extra Deck to Xyz Summon
// over Jacks, copies in the hand and Graveyard to Set after a K9 monster battles, and an opponent's monster to battle or destroy.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, GRAVE),
    (ID, GRAVE),
    (67515699, MZONE),
    (92248362, MZONE),
    (92248362, MZONE),
    (27420823, EXTRA),
    (90303227, EXTRA),
    (26202165, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON + CATEGORY_DESTROY);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_hint_timing(duel, 0, TIMING_MAIN_END | TIMINGS_CHECK_MONSTER);
    e1.set_count_limit(duel, 1, ID);
    e1.set_condition(duel, |duel, _| duel.is_main_phase());
    e1.set_target(duel, target);
    e1.set_operation(duel, activate);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_SET);
    e2.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e2.set_code(duel, EVENT_PHASE | PHASE_BATTLE);
    e2.set_range(duel, LOCATION_GRAVE);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_condition(duel, |duel, a| duel.has_flag_effect(a.tp, ID));
    e2.set_target(duel, settg);
    e2.set_operation(duel, setop);
    c.register_effect(duel, e2);
    aux::global_check(duel, ID, |duel| {
        let ge1 = duel.create_effect(c);
        ge1.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_CONTINUOUS);
        ge1.set_code(duel, EVENT_BATTLED);
        ge1.set_operation(duel, checkop);
        duel.register_effect(ge1, 0);
    });
}

fn checkop(duel: &mut Duel, _: &Args) -> Script<()> {
    let (bc0, bc1) = duel.get_battle_monster(0);
    for (tp, bc) in [(0, bc0), (1, bc1)] {
        if bc.is_some_and(|bc| bc.is_set_card(duel, &[SET_K9])) {
            duel.register_flag_effect(tp, ID, RESET_PHASE | PHASE_END, 0, 1);
        }
    }
    Ok(())
}

fn tgfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    let pg = aux::get_must_be_material_group(duel, tp, Some(c.into()), tp, None, None, REASON_XYZ);
    (pg.is_empty() || (pg.len() == 1 && pg.contains(c)))
        && c.is_set_card(duel, &[SET_K9])
        && c.is_type(duel, TYPE_XYZ)
        && c.is_faceup(duel)
        && duel.is_existing_matching_card(
            |sc, duel| spfilter(sc, duel, e, tp, c, c.get_code(duel)),
            tp,
            LOCATION_EXTRA,
            0,
            1,
            None,
        )
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8, mc: CardId, code: u32) -> bool {
    mc.is_type_full(duel, TYPE_XYZ, Some(c), SUMMON_TYPE_XYZ.into(), tp)
        && c.is_set_card(duel, &[SET_K9])
        && !c.is_code(duel, code)
        && c.is_type(duel, TYPE_XYZ)
        && duel.get_location_count_from_ex(tp, tp, Some(mc.into()), c) > 0
        && mc.is_can_be_xyz_material(duel, Some(c), tp, REASON_XYZ | REASON_MATERIAL)
        && c.is_can_be_special_summoned(duel, e, SUMMON_TYPE_XYZ, tp, false, false)
}

fn target(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if let Some(chkc) = chkc {
        return Ok(chkc.is_controler(duel, tp) && chkc.is_location(duel, LOCATION_MZONE) && tgfilter(chkc, duel, e, tp));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(|c, duel| tgfilter(c, duel, e, tp), tp, LOCATION_MZONE, 0, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TARGET);
    duel.select_target(tp, |c, duel| tgfilter(c, duel, e, tp), tp, LOCATION_MZONE, 0, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, tp, LOCATION_EXTRA as i32);
    Ok(false)
}

fn activate(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let Some(tc) = duel.get_first_target() else { return Ok(()) };
    let pg = aux::get_must_be_material_group(duel, tp, Some(tc.into()), tp, None, None, REASON_XYZ);
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
    let code = tc.get_code(duel);
    let g = duel.select_matching_card(tp, |c, duel| spfilter(c, duel, e, tp, tc, code), tp, LOCATION_EXTRA, 0, 1, 1, None)?;
    let Some(sc) = g.get_first() else { return Ok(()) };
    sc.set_material(duel, Some(tc.into()));
    duel.overlay(sc, tc)?;
    if duel.special_summon(sc, SUMMON_TYPE_XYZ, tp, tp, false, false, POS_FACEUP)? == 0 {
        return Ok(());
    }
    sc.complete_procedure(duel);
    if duel.get_field_group_count(tp, 0, LOCATION_ONFIELD) > 0 && duel.select_yes_no(tp, aux::stringid(ID, 2))? {
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_DESTROY);
        let g = duel.select_matching_card(tp, aux::TRUE, tp, 0, LOCATION_ONFIELD, 1, 1, None)?;
        if !g.is_empty() {
            duel.hint_selection(g);
            duel.break_effect()?;
            duel.destroy(g, REASON_EFFECT)?;
        }
    }
    Ok(())
}

fn settg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let c = a.e.get_handler(duel);
    if chk == 0 {
        return Ok(c.is_ssetable(duel));
    }
    duel.set_operation_info(0, CATEGORY_LEAVE_GRAVE, Some(c.into()), 1, a.tp, 0);
    Ok(false)
}

fn setop(duel: &mut Duel, a: &Args) -> Script<()> {
    let c = a.e.get_handler(duel);
    if c.is_relate_to_effect(duel, a.e) && c.is_ssetable(duel) {
        duel.sset(a.tp, c)?;
    }
    Ok(())
}
