// Melffy Pinny
use crate::prelude::*;
use crate::testing::{Check, GRAVE, HAND, MZONE, OPPONENT, SZONE};

const ID: u32 = 34800281;

pub const LISTED_SERIES: &[u16] = &[SET_MELFFY];
pub const LISTED_NAMES: &[u32] = &[ID];
// Copies in the hand with Melffy Rabby, a Beast Normal Monster, on the field for the opponent's Compulsory Evacuation Device to
// return, and in the Graveyard with Number 29: Mannequin Cat, a Beast Xyz Monster, to attach to.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, HAND),
    (ID, GRAVE),
    (20129614, MZONE),
    (20129614, MZONE),
    (54191698, MZONE),
    (94192409, SZONE | OPPONENT),
    (94192409, SZONE | OPPONENT),
    (94192409, SZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e1.set_type(duel, EFFECT_TYPE_QUICK_O);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_range(duel, LOCATION_HAND);
    e1.set_count_limit(duel, 1, (ID, 0));
    e1.set_condition(duel, |duel, a| duel.has_flag_effect(a.tp, ID));
    e1.set_target(duel, sptg);
    e1.set_operation(duel, spop);
    e1.set_hint_timing(duel, 0, TIMING_STANDBY_PHASE | TIMING_MAIN_END | TIMINGS_CHECK_MONSTER_E);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_LEAVE_GRAVE);
    e2.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e2.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e2.set_code(duel, EVENT_PHASE + PHASE_END);
    e2.set_range(duel, LOCATION_GRAVE);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_condition(duel, |duel, a| duel.is_turn_player(a.tp));
    e2.set_target(duel, attachtg);
    e2.set_operation(duel, attachop);
    c.register_effect(duel, e2);
    aux::global_check(duel, ID, |duel| {
        let ge1 = duel.create_effect(c);
        ge1.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_CONTINUOUS);
        ge1.set_code(duel, EVENT_TO_HAND);
        ge1.set_operation(duel, regop);
        duel.register_effect(ge1, 0);
    });
}

fn regop(duel: &mut Duel, a: &Args) -> Script<()> {
    for ec in a.eg.unwrap_or_default().iter() {
        if ec.is_previous_location(duel, LOCATION_MZONE)
            && ec.is_previous_position(duel, POS_FACEUP)
            && ec.is_previous_race_on_field(duel, RACE_BEAST)
            && !ec.is_previous_code_on_field(duel, &[ID])
            && ec.is_race(duel, RACE_BEAST)
        {
            duel.register_flag_effect(ec.get_previous_controler(duel), ID, RESET_PHASE | PHASE_END, 0, 1);
        }
    }
    Ok(())
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if chk == 0 {
        return Ok(duel.get_location_count(tp, LOCATION_MZONE) > 0 && c.is_can_be_special_summoned(duel, e, 0, tp, false, false));
    }
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, Some(c.into()), 1, tp, 0);
    duel.set_possible_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, tp, LOCATION_EXTRA as i32);
    Ok(false)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if !c.is_relate_to_effect(duel, e) || duel.special_summon(c, 0, tp, tp, false, false, POS_FACEUP)? == 0 {
        return Ok(());
    }
    let mg = duel.get_matching_group(|c, duel| c.is_set_card(duel, &[SET_MELFFY]), tp, LOCATION_HAND, 0, None);
    if mg.is_empty() {
        return Ok(());
    }
    let materials = mg.union(c.into());
    let sg = duel.get_matching_group(
        |sc, duel| sc.is_synchro_summonable(duel, Some(c.into()), Some(materials)),
        tp,
        LOCATION_EXTRA,
        0,
        None,
    );
    if sg.is_empty() || !duel.select_yes_no(tp, aux::stringid(ID, 2))? {
        return Ok(());
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
    if let Some(sc) = sg.select(duel, tp, 1, 1, None)?.get_first() {
        duel.break_effect()?;
        duel.synchro_summon(tp, sc, Some(c.into()), Some(materials))?;
    }
    Ok(())
}

fn xyzfilter(c: CardId, duel: &Duel, ac: CardId, tp: u8) -> bool {
    c.is_race(duel, RACE_BEAST)
        && c.is_xyz_monster(duel)
        && c.is_faceup(duel)
        && ac.is_can_be_xyz_material(duel, Some(c), tp, REASON_EFFECT)
}

fn attachtg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (c, tp) = (a.e.get_handler(duel), a.tp);
    if let Some(chkc) = chkc {
        return Ok(chkc.is_controler(duel, tp) && chkc.is_location(duel, LOCATION_MZONE) && xyzfilter(chkc, duel, c, tp));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(|tc, duel| xyzfilter(tc, duel, c, tp), tp, LOCATION_MZONE, 0, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TARGET);
    duel.select_target(tp, |tc, duel| xyzfilter(tc, duel, c, tp), tp, LOCATION_MZONE, 0, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_LEAVE_GRAVE, Some(c.into()), 1, tp, 0);
    Ok(false)
}

fn attachop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if let Some(tc) = duel.get_first_target()
        && c.is_relate_to_effect(duel, e)
        && tc.is_relate_to_effect(duel, e)
        && !tc.is_immune_to_effect(duel, e)
        && c.is_can_be_xyz_material(duel, Some(tc), tp, REASON_EFFECT)
    {
        duel.overlay(tc, c)?;
    }
    Ok(())
}
