// Mind Control
use crate::prelude::*;
use crate::testing::{HAND, MZONE, OPPONENT};

const ID: u32 = 37520316;

// In the hand to Activate, with monsters on the opponent's field to take.
pub const SUPPORT: &[(u32, u16)] = &[(ID, HAND), (14851496, MZONE | OPPONENT), (4042268, MZONE | OPPONENT)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_category(duel, CATEGORY_CONTROL);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_target(duel, target);
    e1.set_operation(duel, activate);
    c.register_effect(duel, e1);
}

fn target(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_MZONE) && chkc.get_controler(duel) != tp && chkc.is_controler_can_be_changed(duel));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(CardId::is_controler_can_be_changed, tp, 0, LOCATION_MZONE, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_CONTROL);
    let g = duel.select_target(tp, CardId::is_controler_can_be_changed, tp, 0, LOCATION_MZONE, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_CONTROL, g, 1, 0, 0);
    Ok(false)
}

fn activate(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if let Some(tc) = duel.get_first_target()
        && tc.is_relate_to_effect(duel, e)
        && duel.get_control_full(tc, tp, PHASE_END as u16, 1, 0xff, duel.get_reason_player())? != 0
    {
        let reset = RESET_EVENT | (RESETS_STANDARD_PHASE_END & !RESET_TURN_SET);
        for (code, value) in [
            (EFFECT_UNRELEASABLE_SUM, Some(1)),
            (EFFECT_UNRELEASABLE_NONSUM, Some(1)),
            (EFFECT_CANNOT_ATTACK, None),
        ] {
            let e1 = duel.create_effect(c);
            e1.set_type(duel, EFFECT_TYPE_SINGLE);
            e1.set_code(duel, code);
            e1.set_reset(duel, reset);
            if let Some(value) = value {
                e1.set_value(duel, value);
            }
            tc.register_effect(duel, e1);
        }
    }
    Ok(())
}
