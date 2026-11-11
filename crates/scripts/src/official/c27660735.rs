// Speedroid Tri-Eyed Dice
use crate::prelude::*;
use crate::testing::{Check, GRAVE, MZONE, OPPONENT};

const ID: u32 = 27660735;

// Elfnote Lucina (13597785) to attack with.
pub const SUPPORT: &[(u32, u16)] = &[(ID, GRAVE), (ID, GRAVE), (13597785, MZONE | OPPONENT), (13597785, MZONE | OPPONENT)];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_type(duel, EFFECT_TYPE_QUICK_O);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_hint_timing(duel, 0, TIMING_ATTACK);
    e1.set_range(duel, LOCATION_GRAVE);
    e1.set_condition(duel, condition);
    e1.set_cost(duel, Cost::self_banish);
    e1.set_operation(duel, operation);
    c.register_effect(duel, e1);
}

fn condition(duel: &Duel, a: &Args) -> bool {
    duel.is_turn_player(1 - a.tp) && (duel.is_able_to_enter_bp() || duel.is_battle_phase())
}

fn operation(duel: &mut Duel, a: &Args) -> Script<()> {
    if duel.get_attacker().is_some() {
        duel.negate_attack()?;
    } else {
        let e1 = duel.create_effect(a.e.get_handler(duel));
        e1.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_CONTINUOUS);
        e1.set_code(duel, EVENT_ATTACK_ANNOUNCE);
        e1.set_reset(duel, RESET_PHASE | PHASE_END);
        e1.set_count_limit(duel, 1, 0);
        e1.set_operation(duel, disop);
        duel.register_effect(e1, a.tp);
    }
    Ok(())
}

fn disop(duel: &mut Duel, _: &Args) -> Script<()> {
    duel.hint(HINT_CARD, 0, ID.into());
    duel.negate_attack()?;
    Ok(())
}
