// Swords of Concealing Light
use crate::prelude::*;
use crate::testing::{HAND, MZONE, OPPONENT};

const ID: u32 = 12923641;

// In the hand, with monsters of the opponent's to turn face-down: Sangan.
pub const SUPPORT: &[(u32, u16)] = &[(ID, HAND), (26202165, MZONE | OPPONENT)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_category(duel, CATEGORY_POSITION + CATEGORY_SET);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_target(duel, target);
    e1.set_operation(duel, activate);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_FIELD);
    e2.set_range(duel, LOCATION_SZONE);
    e2.set_code(duel, EFFECT_CANNOT_CHANGE_POSITION);
    e2.set_property(duel, EFFECT_FLAG_SET_AVAILABLE);
    e2.set_target_range(duel, 0, LOCATION_MZONE);
    c.register_effect(duel, e2);
}

fn target(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(true);
    }
    let (tp, c) = (a.tp, a.e.get_handler(duel));
    c.set_turn_counter(duel, 0);
    let g = duel.get_matching_group(CardId::is_can_turn_set, tp, 0, LOCATION_MZONE, None);
    duel.set_operation_info(0, CATEGORY_POSITION, Some(g), g.len() as u8, tp, POS_FACEDOWN_DEFENSE as i32);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_CONTINUOUS);
    e1.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE);
    e1.set_code(duel, EVENT_PHASE | PHASE_STANDBY);
    e1.set_count_limit(duel, 1, 0);
    e1.set_range(duel, LOCATION_SZONE);
    e1.set_condition(duel, |duel, a| duel.is_turn_player(a.tp));
    e1.set_operation(duel, desop);
    e1.set_reset_full(
        duel,
        RESET_EVENT | RESETS_STANDARD | RESET_PHASE | PHASE_STANDBY | RESET_SELF_TURN,
        2,
    );
    c.register_effect(duel, e1);
    Ok(false)
}

fn activate(duel: &mut Duel, a: &Args) -> Script<()> {
    if !a.e.get_handler(duel).is_relate_to_effect(duel, a.e) {
        return Ok(());
    }
    let g = duel.get_matching_group(CardId::is_can_turn_set, a.tp, 0, LOCATION_MZONE, None);
    if !g.is_empty() {
        duel.change_position(g, POS_FACEDOWN_DEFENSE)?;
    }
    Ok(())
}

fn desop(duel: &mut Duel, a: &Args) -> Script<()> {
    let c = a.e.get_handler(duel);
    let ct = c.get_turn_counter(duel) + 1;
    c.set_turn_counter(duel, ct);
    if ct == 2 {
        duel.destroy(c, REASON_RULE)?;
    }
    Ok(())
}
