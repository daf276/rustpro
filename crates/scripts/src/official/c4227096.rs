// Mistaken Arrest
use crate::prelude::*;

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_operation(duel, activate);
    c.register_effect(duel, e1);
}

fn activate(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    let e1 = duel.create_effect(a.e.get_handler(duel));
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_code(duel, EFFECT_CANNOT_TO_HAND);
    e1.set_target_range(duel, LOCATION_DECK, LOCATION_DECK);
    if duel.is_turn_player(tp) {
        e1.set_reset_full(duel, RESET_PHASE | PHASE_END | RESET_SELF_TURN, 2);
    } else {
        e1.set_reset(duel, RESET_PHASE | PHASE_END | RESET_SELF_TURN);
    }
    duel.register_effect(e1, tp);
    Ok(())
}
