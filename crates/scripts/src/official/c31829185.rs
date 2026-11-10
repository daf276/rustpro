// Dark Necrofear
use crate::prelude::*;
use crate::testing::{Check, GRAVE, HAND, MZONE, OPPONENT};
use engine::constants::MSG_EQUIP;

const ID: u32 = 31829185;
const HEADLESS_KNIGHT: u32 = 5434080;
const ZURE: u32 = 7459013;

// Two copies in the hand and one on the field, with Fiend Normal Monsters in the Graveyard to banish, and Raigeki and a Sangan
// on the opponent's side to destroy the copy and then be equipped.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, HAND),
    (ID, MZONE),
    (HEADLESS_KNIGHT, GRAVE),
    (HEADLESS_KNIGHT, GRAVE),
    (ZURE, GRAVE),
    (ZURE, GRAVE),
    (12580477, HAND | OPPONENT),
    (26202165, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[Check::SpecialSummoned, Check::Message(MSG_EQUIP)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_code(duel, EFFECT_SPSUMMON_PROC);
    e1.set_property(duel, EFFECT_FLAG_UNCOPYABLE);
    e1.set_range(duel, LOCATION_HAND);
    e1.set_condition(duel, spcon);
    e1.set_target(duel, sptg);
    e1.set_operation(duel, spop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_CONTINUOUS);
    e2.set_code(duel, EVENT_TO_GRAVE);
    e2.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE);
    e2.set_operation(duel, tgop);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_F);
    e3.set_code(duel, EVENT_PHASE + PHASE_END);
    e3.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e3.set_range(duel, LOCATION_GRAVE);
    e3.set_count_limit(duel, 1, 0);
    e3.set_condition(duel, |duel, a| a.e.get_handler(duel).get_flag_effect(duel, ID) > 0);
    e3.set_target(duel, eqtg);
    e3.set_operation(duel, eqop);
    c.register_effect(duel, e3);
}

fn spfilter(c: CardId, duel: &Duel) -> bool {
    c.is_race(duel, RACE_FIEND) && c.is_able_to_remove_as_cost(duel) && aux::sp_elim_filter(c, duel, true, false)
}

fn spcon(duel: &Duel, a: &Args) -> bool {
    let Some(c) = a.c else { return true };
    let tp = c.get_controler(duel);
    let rg = duel.get_matching_group(spfilter, tp, LOCATION_MZONE | LOCATION_GRAVE, 0, None);
    duel.get_location_count(tp, LOCATION_MZONE) > -3
        && rg.len() > 2
        && aux::select_unselect_group_check(duel, rg, 3, 3, &aux::chkf_mmz(tp, 1))
}

fn sptg(duel: &mut Duel, a: &Args, _: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let rg = duel.get_matching_group(spfilter, tp, LOCATION_MZONE | LOCATION_GRAVE, 0, None);
    let g = aux::select_unselect_group(duel, rg, 3, 3, &aux::chkf_mmz(tp, 1), tp, HINTMSG_REMOVE)?;
    if g.is_empty() {
        return Ok(false);
    }
    e.set_label_object_group(duel, Some(g));
    Ok(true)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let Some(g) = a.e.get_label_object_group(duel) else { return Ok(()) };
    duel.remove(g, POS_FACEUP, REASON_COST)?;
    Ok(())
}

fn tgop(duel: &mut Duel, a: &Args) -> Script<()> {
    let c = a.e.get_handler(duel);
    if c.is_previous_controler(duel, a.tp) && c.is_previous_location(duel, LOCATION_MZONE) && a.rp != a.tp && a.r & REASON_DESTROY != 0 {
        c.register_flag_effect(duel, ID, RESETS_STANDARD_PHASE_END, 0, 1);
    }
    Ok(())
}

fn eqtg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_MZONE) && aux::check_steal_equip(chkc, duel, e, tp));
    }
    if chk == 0 {
        return Ok(true);
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_CONTROL);
    let g = duel.select_target(
        tp,
        |c, duel| aux::check_steal_equip(c, duel, e, tp),
        tp,
        0,
        LOCATION_MZONE,
        1,
        1,
        None,
    )?;
    let c = e.get_handler(duel);
    duel.set_operation_info(0, CATEGORY_CONTROL, g, 1, 0, 0);
    duel.set_operation_info(0, CATEGORY_EQUIP, Some(c.into()), 1, 0, 0);
    duel.set_operation_info(0, CATEGORY_LEAVE_GRAVE, Some(c.into()), 1, 0, 0);
    Ok(false)
}

fn eqop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if let Some(tc) = duel.get_first_target()
        && c.is_relate_to_effect(duel, e)
        && aux::check_steal_equip(tc, duel, e, tp)
        && tc.is_relate_to_effect(duel, e)
        && duel.equip(tp, c, tc)?
    {
        let e1 = duel.create_effect(tc);
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_code(duel, EFFECT_EQUIP_LIMIT);
        e1.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE);
        e1.set_reset(duel, RESET_EVENT | RESETS_STANDARD);
        e1.set_value(duel, Value::Card(|duel, e, c| i32::from(e.get_owner(duel) == c)));
        c.register_effect(duel, e1);
        let e2 = duel.create_effect(c);
        e2.set_type(duel, EFFECT_TYPE_EQUIP);
        e2.set_code(duel, EFFECT_SET_CONTROL);
        e2.set_value(duel, i32::from(tp));
        e2.set_reset(duel, RESET_EVENT | (RESETS_STANDARD & !RESET_TURN_SET));
        c.register_effect(duel, e2);
    }
    Ok(())
}
