// Rikka Sheet
use crate::prelude::*;
use crate::testing::{Check, HAND, MZONE, SZONE};

const ID: u32 = 68941332;

// Plants (Snakeyashi) on the field to Tribute, and a copy Set to activate on the opponent's turn.
pub const SUPPORT: &[(u32, u16)] = &[(ID, HAND), (ID, SZONE), (29802344, MZONE), (29802344, MZONE)];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_CONTROL);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_hint_timing(duel, 0, TIMINGS_CHECK_MONSTER);
    e1.set_count_limit_full(duel, 1, ID, EFFECT_COUNT_CODE_OATH);
    e1.set_cost(duel, cost);
    e1.set_target(duel, target);
    e1.set_operation(duel, operation);
    e1.set_label(duel, 1);
    c.register_effect(duel, e1);
}

fn filter(c: CardId, duel: &Duel) -> bool {
    c.is_faceup(duel) && (c.is_type(duel, TYPE_EFFECT) || c.get_original_type(duel) & TYPE_EFFECT > 0)
}

fn control_filter(c: CardId, duel: &Duel) -> bool {
    filter(c, duel) && c.is_controler_can_be_changed_full(duel, true, 0xff)
}

fn cfilter(c: CardId, duel: &Duel, tp: u8) -> bool {
    c.is_rikka_releasable(duel, tp)
        && duel.get_mzone_count_full(tp, Some(c.into()), tp, LOCATION_REASON_CONTROL, 0xff) > 0
        && duel.is_existing_target(control_filter, tp, 0, LOCATION_MZONE, 1, Some(c.into()))
}

fn cost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let a_exists = duel.is_existing_target(filter, tp, 0, LOCATION_MZONE, 1, None);
    let b_exists = duel.check_release_group_cost(tp, |c, duel| cfilter(c, duel, tp), (1, 1), false, None, None);
    if chk == 0 {
        return Ok(a_exists || b_exists);
    }
    if b_exists && duel.select_yes_no(tp, aux::stringid(ID, 1))? {
        let g = duel.select_release_group_cost(tp, |c, duel| cfilter(c, duel, tp), (1, 1), false, None, None)?;
        duel.release(g, REASON_COST)?;
        e.set_label(duel, 1);
    } else {
        e.set_label(duel, 0);
    }
    Ok(false)
}

fn target(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_MZONE) && chkc.is_controler(duel, 1 - tp) && filter(chkc, duel));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(filter, tp, 0, LOCATION_MZONE, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_FACEUP);
    if e.get_label(duel) == 1 {
        let g = duel.select_target(tp, control_filter, tp, 0, LOCATION_MZONE, 1, 1, None)?;
        duel.set_operation_info(0, CATEGORY_CONTROL, g, 1, 0, 0);
    } else {
        duel.select_target(tp, filter, tp, 0, LOCATION_MZONE, 1, 1, None)?;
    }
    Ok(false)
}

fn operation(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let Some(tc) = duel.get_first_target() else { return Ok(()) };
    if !(tc.is_faceup(duel) && tc.is_relate_to_effect(duel, e) && !tc.is_immune_to_effect(duel, e)) {
        return Ok(());
    }
    let c = e.get_handler(duel);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 2));
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_code(duel, EFFECT_CANNOT_TRIGGER);
    e1.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE + EFFECT_FLAG_CLIENT_HINT);
    e1.set_reset(duel, RESETS_STANDARD_PHASE_END);
    tc.register_effect(duel, e1);
    if e.get_label(duel) == 1 && tc.is_controler_can_be_changed(duel) && tc.is_controler(duel, 1 - tp) {
        duel.break_effect()?;
        let chose_player = duel.get_reason_player();
        duel.get_control_full(tc, tp, PHASE_END as u16, 1, 0xff, chose_player)?;
        if tc.is_controler(duel, 1 - tp) {
            return Ok(());
        }
        let e2 = duel.create_effect(c);
        e2.set_type(duel, EFFECT_TYPE_SINGLE);
        e2.set_code(duel, EFFECT_CHANGE_RACE);
        e2.set_value(duel, RACE_PLANT as i32);
        e2.set_reset(duel, RESETS_STANDARD_PHASE_END);
        tc.register_effect(duel, e2);
    }
    Ok(())
}
