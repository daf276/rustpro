// Forbidden Chalice
use crate::prelude::*;

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_category(duel, CATEGORY_ATKCHANGE + CATEGORY_DISABLE);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_hint_timing(duel, TIMING_DAMAGE_STEP, TIMING_DAMAGE_STEP | TIMINGS_CHECK_MONSTER);
    e1.set_property(duel, EFFECT_FLAG_CARD_TARGET + EFFECT_FLAG_DAMAGE_STEP);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_condition(duel, aux::stat_change_damage_step_condition);
    e1.set_target(duel, target);
    e1.set_operation(duel, activate);
    c.register_effect(duel, e1);
}

fn target(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_MZONE) && chkc.is_faceup(duel));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(CardId::is_faceup, tp, LOCATION_MZONE, LOCATION_MZONE, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_FACEUP);
    let g = duel.select_target(tp, CardId::is_faceup, tp, LOCATION_MZONE, LOCATION_MZONE, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_DISABLE, g, 1, 0, 0);
    Ok(false)
}

fn activate(duel: &mut Duel, a: &Args) -> Script<()> {
    let c = a.e.get_handler(duel);
    let Some(tc) = duel
        .get_first_target()
        .filter(|tc| tc.is_relate_to_effect(duel, a.e) && tc.is_faceup(duel))
    else {
        return Ok(());
    };
    duel.negate_related_chain(tc, RESET_TURN_SET);
    for (code, value) in [
        (EFFECT_UPDATE_ATTACK, 400),
        (EFFECT_DISABLE, 0),
        (EFFECT_DISABLE_EFFECT, RESET_TURN_SET as i32),
    ] {
        let e1 = duel.create_effect(c);
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_code(duel, code);
        e1.set_reset(duel, RESETS_STANDARD_PHASE_END);
        if value != 0 {
            e1.set_value(duel, value);
        }
        tc.register_effect(duel, e1);
    }
    Ok(())
}
