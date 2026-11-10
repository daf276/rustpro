// Enemy Controller
use crate::prelude::*;
use crate::testing::{HAND, MZONE, OPPONENT};

const ID: u32 = 98045062;

// In the hand, with a monster of its own to Tribute and the opponent's monsters to take or turn.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (35565537, MZONE),
    (26202165, MZONE | OPPONENT),
    (26202165, MZONE | OPPONENT),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_hint_timing(duel, TIMING_BATTLE_PHASE | TIMING_STANDBY_PHASE, TIMING_BATTLE_PHASE);
    e1.set_cost(duel, cost);
    e1.set_target(duel, target);
    e1.set_operation(duel, activate);
    c.register_effect(duel, e1);
}

fn cost(duel: &mut Duel, a: &Args, _: i32) -> Script<bool> {
    a.e.set_label(duel, 9);
    Ok(true)
}

fn controllable(c: CardId, duel: &Duel) -> bool {
    c.is_faceup(duel) && c.is_controler_can_be_changed(duel)
}

fn cfilter(c: CardId, duel: &Duel, tp: u8) -> bool {
    duel.get_mzone_count_full(tp, Some(c.into()), tp, LOCATION_REASON_CONTROL, 0xff) > 0
        && duel.is_existing_target(
            |tc, duel| tc.is_faceup(duel) && tc.is_controler_can_be_changed_full(duel, true, 0xff),
            tp,
            0,
            LOCATION_MZONE,
            1,
            Some(c.into()),
        )
}

fn target(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let label = e.get_label(duel);
    if let Some(chkc) = chkc {
        if !(chkc.is_location(duel, LOCATION_MZONE) && chkc.is_faceup(duel) && chkc.is_controler(duel, 1 - tp)) {
            return Ok(false);
        }
        return Ok(if label == 1 {
            chkc.is_can_change_position(duel)
        } else {
            chkc.is_controler_can_be_changed(duel)
        });
    }
    let movable = aux::faceup_filter(CardId::is_can_change_position);
    let b1 = duel.is_existing_target(movable, tp, 0, LOCATION_MZONE, 1, None);
    let b2 = if label == 9 {
        duel.check_release_group_cost(tp, |c, duel| cfilter(c, duel, tp), (1, 1), false, None, None)
    } else {
        duel.get_location_count_full(tp, LOCATION_MZONE, tp, LOCATION_REASON_CONTROL, 0xff) > 0
            && duel.is_existing_target(controllable, tp, 0, LOCATION_MZONE, 1, None)
    };
    if chk == 0 {
        e.set_label(duel, 0);
        return Ok(b1 || b2);
    }
    let op = duel.select_effect(tp, &[(b1, aux::stringid(ID, 1)), (b2, aux::stringid(ID, 2))])?;
    if op == Some(1) {
        e.set_category(duel, CATEGORY_POSITION);
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_POSCHANGE);
        let movable = aux::faceup_filter(CardId::is_can_change_position);
        let g = duel.select_target(tp, movable, tp, 0, LOCATION_MZONE, 1, 1, None)?;
        duel.set_operation_info(0, CATEGORY_POSITION, g, 1, 0, 0);
    } else {
        e.set_category(duel, CATEGORY_CONTROL);
        if label == 9 {
            let rg = duel.select_release_group_cost(tp, |c, duel| cfilter(c, duel, tp), (1, 1), false, None, None)?;
            duel.release(rg, REASON_COST)?;
        }
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_CONTROL);
        let g = duel.select_target(tp, controllable, tp, 0, LOCATION_MZONE, 1, 1, None)?;
        duel.set_operation_info(0, CATEGORY_CONTROL, g, 1, 0, 0);
    }
    e.set_label(duel, op.map_or(0, |op| op as i64));
    Ok(false)
}

fn activate(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let Some(tc) = duel.get_first_target() else { return Ok(()) };
    if !(tc.is_relate_to_effect(duel, e) && tc.is_faceup(duel) && tc.is_controler(duel, 1 - tp)) {
        return Ok(());
    }
    if e.get_label(duel) == 1 {
        let positions = [POS_FACEUP_DEFENSE, 0, POS_FACEUP_ATTACK, 0];
        duel.change_position_full(tc, positions, false)?;
    } else {
        duel.get_control_full(tc, tp, PHASE_END as u16, 1, 0xff, duel.get_reason_player())?;
    }
    Ok(())
}
