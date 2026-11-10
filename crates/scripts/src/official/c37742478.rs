// Honest
use crate::prelude::*;
use crate::testing::{Check, HAND, MZONE, OPPONENT};

const ID: u32 = 37742478;

// In the hand and on the field, with Maiden of the Moonlight, a LIGHT Normal Monster, to battle the monsters of the opponent.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, MZONE),
    (79629370, MZONE),
    (79629370, MZONE),
    (89631139, MZONE | OPPONENT),
    (89631139, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_TOHAND);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_target(duel, target1);
    e1.set_operation(duel, operation1);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_QUICK_O);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_ATKCHANGE);
    e2.set_code(duel, EVENT_FREE_CHAIN);
    e2.set_hint_timing(duel, TIMING_DAMAGE_STEP, TIMING_DAMAGE_STEP);
    e2.set_range(duel, LOCATION_HAND);
    e2.set_property(duel, EFFECT_FLAG_DAMAGE_STEP);
    e2.set_condition(duel, condition2);
    e2.set_cost(duel, Cost::self_to_grave);
    e2.set_operation(duel, operation2);
    c.register_effect(duel, e2);
}

fn target1(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let c = a.e.get_handler(duel);
    if chk == 0 {
        return Ok(c.is_able_to_hand(duel));
    }
    duel.set_operation_info(0, CATEGORY_TOHAND, Some(c.into()), 1, 0, 0);
    Ok(false)
}

fn operation1(duel: &mut Duel, a: &Args) -> Script<()> {
    let c = a.e.get_handler(duel);
    if c.is_faceup(duel) && c.is_relate_to_effect(duel, a.e) {
        duel.send_to_hand(c, PLAYER_NONE, REASON_EFFECT)?;
    }
    Ok(())
}

fn light_battler(c: CardId, duel: &Duel, tp: u8) -> bool {
    c.is_controler(duel, tp) && c.is_attribute(duel, ATTRIBUTE_LIGHT) && c.is_relate_to_battle(duel)
}

fn condition2(duel: &Duel, a: &Args) -> bool {
    if duel.get_current_phase() != PHASE_DAMAGE || duel.is_damage_calculated() {
        return false;
    }
    let (Some(at), Some(d)) = (duel.get_attacker(), duel.get_attack_target()) else {
        return false;
    };
    d.is_faceup(duel) && (light_battler(at, duel, a.tp) || light_battler(d, duel, a.tp))
}

fn operation2(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    let (at, d) = (duel.get_attacker(), duel.get_attack_target());
    unported!(duel, at.is_none() || d.is_none());
    let (Some(at), Some(d)) = (at, d) else { return Ok(()) };
    if !at.is_relate_to_battle(duel) || !d.is_relate_to_battle(duel) {
        return Ok(());
    }
    let e1 = duel.create_effect(a.e.get_handler(duel));
    e1.set_owner_player(duel, tp);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_code(duel, EFFECT_UPDATE_ATTACK);
    e1.set_reset(duel, RESETS_STANDARD_PHASE_END);
    let (receiver, source) = if at.is_controler(duel, tp) { (at, d) } else { (d, at) };
    e1.set_value(duel, source.get_attack(duel));
    receiver.register_effect(duel, e1);
    Ok(())
}
