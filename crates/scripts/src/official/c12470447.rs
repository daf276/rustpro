// Curse of Fiend
use crate::prelude::*;
use crate::testing::{Check, HAND, MZONE, OPPONENT, SZONE};

const ID: u32 = 12470447;

// In the hand and Set to activate in the Standby Phase, with monsters on both sides to change position.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, SZONE),
    (ID, SZONE),
    (15025844, MZONE),
    (15025844, MZONE),
    (15025844, MZONE | OPPONENT),
    (15025844, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(0)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_category(duel, CATEGORY_POSITION);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_hint_timing(duel, TIMING_STANDBY_PHASE, TIMING_STANDBY_PHASE);
    e1.set_condition(duel, |duel, _| duel.is_phase(PHASE_STANDBY));
    e1.set_target(duel, target);
    e1.set_operation(duel, activate);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_SINGLE);
    e2.set_property(duel, EFFECT_FLAG_UNCOPYABLE | EFFECT_FLAG_CANNOT_DISABLE);
    e2.set_code(duel, EFFECT_BECOME_QUICK);
    e2.set_range(duel, LOCATION_SZONE);
    e2.set_condition(duel, condition);
    c.register_effect(duel, e2);
}

fn condition(duel: &Duel, a: &Args) -> bool {
    let c = a.e.get_handler(duel);
    duel.is_turn_player(a.e.get_handler_player(duel))
        && c.is_location(duel, LOCATION_SZONE)
        && duel.is_phase(PHASE_STANDBY)
        && duel.get_current_chain() == 0
}

fn target(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(CardId::is_can_change_position, tp, LOCATION_MZONE, LOCATION_MZONE, 1, None));
    }
    let g = duel.get_matching_group(CardId::is_can_change_position, tp, LOCATION_MZONE, LOCATION_MZONE, None);
    duel.set_operation_info(0, CATEGORY_POSITION, Some(g), g.len() as u8, 0, 0);
    Ok(false)
}

fn activate(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    let g = duel.get_matching_group(CardId::is_can_change_position, tp, LOCATION_MZONE, LOCATION_MZONE, None);
    if !g.is_empty() {
        duel.change_position_full(
            g,
            [POS_FACEUP_DEFENSE, POS_FACEDOWN_DEFENSE, POS_FACEUP_ATTACK, POS_FACEUP_ATTACK],
            false,
        )?;
    }
    let e1 = duel.create_effect(a.e.get_handler(duel));
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_property(duel, EFFECT_FLAG_SET_AVAILABLE);
    e1.set_code(duel, EFFECT_CANNOT_CHANGE_POSITION);
    e1.set_target_range(duel, LOCATION_MZONE, LOCATION_MZONE);
    e1.set_reset(duel, RESET_PHASE | PHASE_END);
    duel.register_effect(e1, tp);
    Ok(())
}
