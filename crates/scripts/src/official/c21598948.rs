// Fairy Box
use crate::prelude::*;
use crate::testing::{Check, HAND, MZONE, OPPONENT, SZONE};

const ID: u32 = 21598948;

pub const TOSS_COIN: bool = true;

// In the hand and Set, against the opponent's Terra the Terribles attacking.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, SZONE),
    (63308047, MZONE | OPPONENT),
    (63308047, MZONE | OPPONENT),
    (63308047, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_hint_timing(duel, 0, TIMING_MAIN_END);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 0));
    e2.set_category(duel, CATEGORY_ATKCHANGE + CATEGORY_COIN);
    e2.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_F);
    e2.set_code(duel, EVENT_ATTACK_ANNOUNCE);
    e2.set_range(duel, LOCATION_SZONE);
    e2.set_condition(duel, |duel, a| {
        duel.get_attacker().is_some_and(|at| at.is_controler(duel, 1 - a.tp))
    });
    e2.set_target(duel, atktg2);
    e2.set_operation(duel, atkop);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_CONTINUOUS);
    e3.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE + EFFECT_FLAG_UNCOPYABLE);
    e3.set_code(duel, EVENT_PHASE | PHASE_STANDBY);
    e3.set_range(duel, LOCATION_SZONE);
    e3.set_count_limit(duel, 1, 0);
    e3.set_condition(duel, |duel, a| duel.is_turn_player(a.tp));
    e3.set_operation(duel, mtop);
    c.register_effect(duel, e3);
}

fn atktg2(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(true);
    }
    if let Some(at) = duel.get_attacker() {
        duel.set_target_card(at);
    }
    duel.set_operation_info(0, CATEGORY_COIN, None, 0, a.tp, 1);
    Ok(false)
}

fn atkop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let Some(at) = duel.get_attacker() else { return Ok(()) };
    if !at.is_relate_to_effect(duel, e) {
        return Ok(());
    }
    if duel.call_coin(tp)? {
        let e1 = duel.create_effect(e.get_handler(duel));
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_code(duel, EFFECT_SET_ATTACK_FINAL);
        e1.set_value(duel, 0);
        e1.set_reset(duel, RESET_EVENT | RESETS_STANDARD | RESET_PHASE | PHASE_BATTLE);
        at.register_effect(duel, e1);
    }
    Ok(())
}

fn mtop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    if duel.check_lp_cost(tp, 500) && duel.select_yes_no(tp, aux::stringid(ID, 1))? {
        duel.pay_lp_cost(tp, 500)?;
    } else {
        duel.destroy(a.e.get_handler(duel), REASON_COST)?;
    }
    Ok(())
}
