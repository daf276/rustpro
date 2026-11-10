// Time Seal
use crate::prelude::*;
use crate::testing::HAND;

const ID: u32 = 35316708;

pub const SUPPORT: &[(u32, u16)] = &[(ID, HAND), (ID, HAND)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_hint_timing(duel, 0, TIMING_END_PHASE);
    e1.set_operation(duel, activate);
    c.register_effect(duel, e1);
}

fn activate(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    let e1 = duel.create_effect(a.e.get_handler(duel));
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_property(duel, EFFECT_FLAG_PLAYER_TARGET);
    e1.set_target_range(duel, 0, 1);
    e1.set_code(duel, EFFECT_SKIP_DP);
    if duel.is_phase(PHASE_DRAW) && duel.is_turn_player(1 - tp) {
        e1.set_reset_full(duel, RESET_PHASE | PHASE_DRAW | RESET_OPPO_TURN, 2);
    } else {
        e1.set_reset(duel, RESET_PHASE | PHASE_DRAW | RESET_OPPO_TURN);
    }
    duel.register_effect(e1, tp);
    Ok(())
}
