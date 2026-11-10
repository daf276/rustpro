// Threatening Roar
use crate::prelude::*;
use crate::testing::{MZONE, OPPONENT, SZONE};

const ID: u32 = 36361633;

// Set, with a monster of the opponent's to attack: Sangan.
pub const SUPPORT: &[(u32, u16)] = &[(ID, SZONE), (26202165, MZONE | OPPONENT)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_hint_timing(duel, 0, TIMING_BATTLE_START);
    e1.set_condition(duel, condition);
    e1.set_operation(duel, activate);
    c.register_effect(duel, e1);
}

fn condition(duel: &Duel, a: &Args) -> bool {
    duel.is_turn_player(1 - a.tp) && duel.get_current_phase() & (PHASE_MAIN2 | PHASE_END) == 0
}

fn activate(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    let e1 = duel.create_effect(a.e.get_handler(duel));
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_code(duel, EFFECT_CANNOT_ATTACK_ANNOUNCE);
    e1.set_property(duel, EFFECT_FLAG_PLAYER_TARGET);
    e1.set_reset(duel, RESET_PHASE | PHASE_END);
    e1.set_target_range(duel, 0, 1);
    duel.register_effect(e1, tp);
    Ok(())
}
