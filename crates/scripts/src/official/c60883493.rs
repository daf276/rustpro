// Vanquish Soul Snow Devil
use crate::prelude::*;
use crate::testing::{Check, HAND, SZONE};

const ID: u32 = 60883493;

pub const LISTED_SERIES: &[u16] = &[SET_VANQUISH_SOUL];
// Set, with DARK, EARTH and FIRE Vanquish Soul monsters in the hand to reveal and summon.
pub const SUPPORT: &[(u32, u16)] = &[(ID, SZONE), (ID, SZONE), (29280200, HAND), (93156774, HAND), (29302858, HAND)];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_DAMAGE + CATEGORY_SPECIAL_SUMMON + CATEGORY_DESTROY);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_hint_timing(duel, 0, TIMING_MAIN_END | TIMINGS_CHECK_MONSTER_E);
    e1.set_count_limit_full(duel, 1, ID, EFFECT_COUNT_CODE_OATH);
    e1.set_cost(duel, cost);
    e1.set_target(duel, target);
    e1.set_operation(duel, activate);
    c.register_effect(duel, e1);
}

fn cfilter(c: CardId, duel: &Duel) -> bool {
    c.is_attribute(duel, ATTRIBUTE_DARK | ATTRIBUTE_EARTH | ATTRIBUTE_FIRE) && !c.is_public(duel)
}

fn cost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    e.set_label(duel, -100);
    let rvg = duel.get_matching_group(cfilter, tp, LOCATION_HAND, 0, None);
    if chk == 0 {
        return Ok(!rvg.is_empty());
    }
    let check = aux::dpcheck(|c, duel| i64::from(c.get_attribute(duel)));
    let g = aux::select_unselect_group(duel, rvg, 1, 3, &check, tp, HINTMSG_CONFIRM)?;
    e.set_label(duel, g.len() as i64);
    duel.confirm_cards(1 - tp, g)?;
    duel.shuffle_hand(tp)?;
    Ok(false)
}

fn target(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if chk == 0 {
        let res = e.get_label(duel) != 0;
        e.set_label(duel, 0);
        return Ok(res);
    }
    let ct = e.get_label(duel);
    let mut dam = 400;
    if ct >= 1 {
        duel.set_possible_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, tp, LOCATION_HAND as i32);
    }
    if ct >= 2 {
        dam += 600;
    }
    if ct == 3 {
        dam += 800;
        let dg = duel.get_matching_group(aux::TRUE, tp, LOCATION_MZONE, LOCATION_MZONE, None);
        duel.set_possible_operation_info(0, CATEGORY_DESTROY, Some(dg), dg.len() as u8, 0, 0);
    }
    duel.set_operation_info(0, CATEGORY_DAMAGE, None, 0, 1 - tp, dam);
    Ok(false)
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_set_card(duel, &[SET_VANQUISH_SOUL]) && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
}

fn activate(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let ct = e.get_label(duel);
    let mut break_eff = false;
    if ct >= 1 && duel.damage(1 - tp, 400, REASON_EFFECT)? > 0 {
        break_eff = true;
        let filter = |c, duel: &Duel| spfilter(c, duel, e, tp);
        if duel.get_location_count(tp, LOCATION_MZONE) > 0
            && duel.is_existing_matching_card(filter, tp, LOCATION_HAND, 0, 1, None)
            && duel.select_yes_no(tp, aux::stringid(ID, 1))?
        {
            duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
            let sg = duel.select_matching_card(tp, filter, tp, LOCATION_HAND, 0, 1, 1, None)?;
            duel.break_effect()?;
            duel.special_summon(sg, 0, tp, tp, false, false, POS_FACEUP)?;
        }
    }
    if ct >= 2 {
        if break_eff {
            duel.break_effect()?;
        }
        duel.damage(1 - tp, 600, REASON_EFFECT)?;
        let c = e.get_handler(duel);
        let e1 = duel.create_effect(c);
        e1.set_type(duel, EFFECT_TYPE_FIELD);
        e1.set_code(duel, EFFECT_INDESTRUCTABLE_EFFECT);
        e1.set_target_range(duel, LOCATION_MZONE, 0);
        e1.set_target_card(duel, |duel, _, c| c.is_set_card(duel, &[SET_VANQUISH_SOUL]));
        e1.set_value(duel, 1);
        e1.set_reset(duel, RESET_PHASE | PHASE_END);
        duel.register_effect(e1, tp);
        aux::register_client_hint(duel, c, 0, tp, 1, 0, Some(aux::stringid(ID, 2)));
        break_eff = true;
    }
    if ct == 3 {
        if break_eff {
            duel.break_effect()?;
        }
        if duel.damage(1 - tp, 800, REASON_EFFECT)? > 0
            && duel.is_existing_matching_card(aux::TRUE, tp, LOCATION_MZONE, LOCATION_MZONE, 1, None)
            && duel.select_yes_no(tp, aux::stringid(ID, 3))?
        {
            let dg = duel.get_matching_group(aux::TRUE, tp, LOCATION_MZONE, LOCATION_MZONE, None);
            duel.break_effect()?;
            duel.destroy(dg, REASON_EFFECT)?;
        }
    }
    Ok(())
}
