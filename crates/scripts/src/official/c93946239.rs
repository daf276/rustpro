// Into the Void
use crate::prelude::*;
use crate::testing::{Check, HAND};

// Three Normal Monsters (Destroyer Golem) in the hand so that it can be activated.
pub const SUPPORT: &[(u32, u16)] = &[(73481154, HAND), (73481154, HAND), (73481154, HAND)];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(0)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_category(duel, CATEGORY_DRAW);
    e1.set_property(duel, EFFECT_FLAG_PLAYER_TARGET);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_condition(duel, |duel, a| duel.get_field_group_count(a.tp, LOCATION_HAND, 0) > 2);
    e1.set_target(duel, target);
    e1.set_operation(duel, activate);
    c.register_effect(duel, e1);
}

fn target(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.is_player_can_draw(tp, 1));
    }
    duel.set_target_player(tp);
    duel.set_target_param(1);
    duel.set_operation_info(0, CATEGORY_DRAW, None, 0, tp, 1);
    Ok(false)
}

fn activate(duel: &mut Duel, a: &Args) -> Script<()> {
    let p: u8 = duel.get_chain_info(0, CHAININFO_TARGET_PLAYER);
    let d: u32 = duel.get_chain_info(0, CHAININFO_TARGET_PARAM);
    duel.draw(p, d, REASON_EFFECT)?;
    let e1 = duel.create_effect(a.e.get_handler(duel));
    e1.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_CONTINUOUS);
    e1.set_code(duel, EVENT_PHASE + PHASE_END);
    e1.set_count_limit(duel, 1, 0);
    e1.set_reset(duel, RESET_PHASE | PHASE_END);
    e1.set_operation(duel, disop);
    duel.register_effect(e1, p);
    Ok(())
}

fn disop(duel: &mut Duel, a: &Args) -> Script<()> {
    let g = duel.get_field_group(a.e.get_owner_player(duel), LOCATION_HAND, 0);
    duel.send_to_grave(g, REASON_EFFECT | REASON_DISCARD)?;
    Ok(())
}
