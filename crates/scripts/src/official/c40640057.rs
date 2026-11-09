// Kuriboh
use crate::prelude::*;
use crate::testing::{Check, HAND, MZONE, OPPONENT};

const ID: u32 = 40640057;

// Copies in the hand to discard against the opponent's monsters attacking.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, HAND),
    (ID, HAND),
    (7892180, MZONE | OPPONENT),
    (59053232, MZONE | OPPONENT),
    (59053232, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_type(duel, EFFECT_TYPE_QUICK_O);
    e1.set_range(duel, LOCATION_HAND);
    e1.set_code(duel, EVENT_PRE_DAMAGE_CALCULATE);
    e1.set_condition(duel, |duel, a| duel.is_turn_player(1 - a.tp) && duel.get_battle_damage(a.tp) > 0);
    e1.set_cost(duel, Cost::self_discard);
    e1.set_operation(duel, op);
    c.register_effect(duel, e1);
}

fn op(duel: &mut Duel, a: &Args) -> Script<()> {
    let e1 = duel.create_effect(a.e.get_handler(duel));
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_code(duel, EFFECT_AVOID_BATTLE_DAMAGE);
    e1.set_property(duel, EFFECT_FLAG_PLAYER_TARGET);
    e1.set_target_range(duel, 1, 0);
    e1.set_reset(duel, RESET_PHASE | PHASE_DAMAGE);
    duel.register_effect(e1, a.tp);
    Ok(())
}
