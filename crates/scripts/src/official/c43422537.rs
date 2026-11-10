// Double Summon
use crate::prelude::*;
use crate::testing::HAND;

const ID: u32 = 43422537;

// In the hand, with a second copy for the case where the first is already active.
pub const SUPPORT: &[(u32, u16)] = &[(ID, HAND), (ID, HAND)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_target(duel, target);
    e1.set_operation(duel, activate);
    c.register_effect(duel, e1);
}

fn target(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk != 0 {
        return Ok(false);
    }
    let value = duel
        .is_player_affected_by_effect(a.tp, EFFECT_SET_SUMMON_COUNT_LIMIT)
        .map(|te| te.get_value(duel));
    unported!(duel, !matches!(value, None | Some(Value::Int(_))));
    Ok(!matches!(value, Some(Value::Int(ct)) if ct >= 2))
}

fn activate(duel: &mut Duel, a: &Args) -> Script<()> {
    let e1 = duel.create_effect(a.e.get_handler(duel));
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_code(duel, EFFECT_SET_SUMMON_COUNT_LIMIT);
    e1.set_property(duel, EFFECT_FLAG_PLAYER_TARGET);
    e1.set_target_range(duel, 1, 0);
    e1.set_value(duel, 2);
    e1.set_reset(duel, RESET_PHASE | PHASE_END);
    duel.register_effect(e1, a.tp);
    Ok(())
}
