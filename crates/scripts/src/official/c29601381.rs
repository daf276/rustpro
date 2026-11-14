// Springans Captain Sargas
use crate::prelude::*;
use crate::testing::{Check, EXTRA, GRAVE, HAND, MZONE, OPPONENT};

const ID: u32 = 29601381;

pub const LISTED_SERIES: &[u16] = &[SET_SPRINGANS];
// Copies in the hand and Graveyard to attach, and on the field to detach, to Springans Merrymaker, Xyz Summoned from Springans Kitt, with Sangan
// on the opponent's side to destroy.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, HAND),
    (ID, GRAVE),
    (48285768, EXTRA),
    (48285768, EXTRA),
    (45484331, MZONE),
    (45484331, MZONE),
    (26202165, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e1.set_range(duel, LOCATION_GRAVE | LOCATION_HAND | LOCATION_MZONE);
    e1.set_count_limit(duel, 1, ID);
    e1.set_target(duel, mattg);
    e1.set_operation(duel, matop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e2.set_type(duel, EFFECT_TYPE_QUICK_O);
    e2.set_code(duel, EVENT_FREE_CHAIN);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_condition(duel, |duel, a| duel.is_turn_player(1 - a.tp));
    e2.set_cost(duel, cost);
    e2.set_target(duel, destg);
    e2.set_operation(duel, stock::destroy_first_target);
    e2.set_hint_timing(duel, 0, TIMING_END_PHASE);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 2));
    e3.set_type(duel, EFFECT_TYPE_XMATERIAL);
    e3.set_property(duel, EFFECT_FLAG_SINGLE_RANGE);
    e3.set_condition(duel, |duel, a| a.e.get_handler(duel).is_set_card(duel, &[SET_SPRINGANS]));
    e3.set_range(duel, LOCATION_MZONE);
    e3.set_code(duel, EFFECT_UPDATE_ATTACK);
    e3.set_value(duel, 500);
    c.register_effect(duel, e3);
}

fn matfilter(c: CardId, duel: &Duel) -> bool {
    c.is_faceup(duel) && c.is_set_card(duel, &[SET_SPRINGANS]) && c.is_type(duel, TYPE_XYZ)
}

fn mattg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_MZONE) && chkc.is_controler(duel, tp) && matfilter(chkc, duel));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(matfilter, tp, LOCATION_MZONE, 0, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TARGET);
    duel.select_target(tp, matfilter, tp, LOCATION_MZONE, 0, 1, 1, None)?;
    let c = e.get_handler(duel);
    if c.is_location(duel, LOCATION_GRAVE) {
        duel.set_operation_info(0, CATEGORY_LEAVE_GRAVE, Some(c.into()), 1, 0, 0);
    }
    Ok(false)
}

fn matop(duel: &mut Duel, a: &Args) -> Script<()> {
    let e = a.e;
    let c = e.get_handler(duel);
    if let Some(tc) = duel.get_first_target()
        && c.is_relate_to_effect(duel, e)
        && tc.is_relate_to_effect(duel, e)
        && !tc.is_immune_to_effect(duel, e)
    {
        duel.overlay(tc, c)?;
    }
    Ok(())
}

pub fn cost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.check_remove_overlay_card(tp, 1, 0, 1, REASON_COST));
    }
    duel.remove_overlay_card(tp, 1, 0, 1, 1, REASON_COST)?;
    Ok(false)
}

fn destg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_on_field(duel) && chkc.is_faceup(duel));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(CardId::is_faceup, tp, LOCATION_ONFIELD, LOCATION_ONFIELD, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_DESTROY);
    let g = duel.select_target(tp, CardId::is_faceup, tp, LOCATION_ONFIELD, LOCATION_ONFIELD, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_DESTROY, g, 1, 0, 0);
    Ok(false)
}
