// Score the Melodious Diva
use crate::prelude::*;
use crate::testing::{HAND, MZONE, OPPONENT};

const ID: u32 = 41767843;

pub const LISTED_SERIES: &[u16] = &[SET_MELODIOUS];

// In the hand, with Aria and Elegy on the field to battle the opponent's Sangan.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, HAND),
    (40502912, MZONE),
    (40502912, MZONE),
    (79514956, MZONE),
    (79514956, MZONE),
    (26202165, MZONE | OPPONENT),
    (26202165, MZONE | OPPONENT),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_ATKCHANGE + CATEGORY_DEFCHANGE);
    e1.set_type(duel, EFFECT_TYPE_QUICK_O);
    e1.set_code(duel, EVENT_PRE_DAMAGE_CALCULATE);
    e1.set_range(duel, LOCATION_HAND);
    e1.set_condition(duel, condition);
    e1.set_cost(duel, Cost::self_to_grave);
    e1.set_operation(duel, operation);
    c.register_effect(duel, e1);
}

fn condition(duel: &Duel, a: &Args) -> bool {
    let (Some(attacker), Some(target)) = (duel.get_attacker(), duel.get_attack_target()) else {
        return false;
    };
    let (attacker, target) = if attacker.is_controler(duel, 1 - a.tp) {
        (target, attacker)
    } else {
        (attacker, target)
    };
    attacker.is_set_card(duel, &[SET_MELODIOUS])
        && attacker.is_relate_to_battle(duel)
        && (target.get_attack(duel) > 0 || target.get_defense(duel) > 0)
}

fn operation(duel: &mut Duel, a: &Args) -> Script<()> {
    let attacker = duel.get_attacker();
    let d = match attacker {
        Some(attacker) if attacker.is_controler(duel, 1 - a.tp) => Some(attacker),
        Some(_) => duel.get_attack_target(),
        None => None,
    };
    // The Lua errors on a missing attacker or attack target.
    unported!(duel, d.is_none());
    let Some(d) = d else { return Ok(()) };
    if !d.is_relate_to_battle(duel) || d.is_facedown(duel) {
        return Ok(());
    }
    let e1 = duel.create_effect(a.e.get_handler(duel));
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_code(duel, EFFECT_SET_ATTACK_FINAL);
    e1.set_reset(duel, RESETS_STANDARD_PHASE_END);
    e1.set_value(duel, 0);
    d.register_effect(duel, e1);
    let e2 = e1.clone_effect(duel);
    e2.set_code(duel, EFFECT_SET_DEFENSE_FINAL);
    d.register_effect(duel, e2);
    Ok(())
}
