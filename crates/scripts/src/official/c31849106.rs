// Different Dimension Ground
use crate::prelude::*;
use crate::testing::HAND;

const ID: u32 = 31849106;

pub const SUPPORT: &[(u32, u16)] = &[(ID, HAND)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_operation(duel, activate);
    e1.set_cost(duel, cost);
    c.register_effect(duel, e1);
}

fn cost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    Ok(chk == 0 && duel.get_flag_effect(a.tp, ID) == 0)
}

fn activate(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    let e1 = duel.create_effect(a.e.get_handler(duel));
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_property(
        duel,
        EFFECT_FLAG_SET_AVAILABLE + EFFECT_FLAG_IGNORE_RANGE + EFFECT_FLAG_IGNORE_IMMUNE,
    );
    e1.set_code(duel, EFFECT_TO_GRAVE_REDIRECT);
    e1.set_target_card(duel, rmtarget);
    e1.set_target_range(duel, 0xff, 0xff);
    e1.set_value(duel, LOCATION_REMOVED as i32);
    e1.set_reset(duel, RESET_PHASE | PHASE_END);
    duel.register_effect(e1, tp);
    duel.register_flag_effect(tp, ID, RESET_PHASE | PHASE_END, 0, 1);
    Ok(())
}

fn rmtarget(duel: &Duel, e: EffectId, c: CardId) -> bool {
    !c.is_location(duel, LOCATION_OVERLAY) && !c.is_spell_trap(duel) && duel.is_player_can_remove(e.get_handler_player(duel), Some(c))
}
