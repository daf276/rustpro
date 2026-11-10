// Injection Fairy Lily
use crate::prelude::*;
use crate::testing::{Check, MZONE, OPPONENT};

const ID: u32 = 79575620;

// Copies on the field to be attacked by the opponent's Sangan and Elfnote Lucina.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (ID, MZONE),
    (ID, MZONE),
    (26202165, MZONE | OPPONENT),
    (13597785, MZONE | OPPONENT),
    (13597785, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_ATKCHANGE);
    e1.set_type(duel, EFFECT_TYPE_QUICK_O);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_code(duel, EVENT_PRE_DAMAGE_CALCULATE);
    e1.set_condition(duel, con);
    e1.set_cost(duel, cost);
    e1.set_operation(duel, op);
    c.register_effect(duel, e1);
}

fn con(duel: &Duel, a: &Args) -> bool {
    let c = a.e.get_handler(duel);
    c.get_flag_effect(duel, ID) == 0 && (duel.get_attacker() == Some(c) || duel.get_attack_target() == Some(c))
}

fn cost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    if chk == 0 {
        return Ok(duel.check_lp_cost(a.tp, 2000));
    }
    duel.pay_lp_cost(a.tp, 2000)?;
    a.e.get_handler(duel)
        .register_flag_effect(duel, ID, RESET_PHASE | PHASE_DAMAGE_CAL, 0, 1);
    Ok(false)
}

fn op(duel: &mut Duel, a: &Args) -> Script<()> {
    let c = a.e.get_handler(duel);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_code(duel, EFFECT_UPDATE_ATTACK);
    e1.set_reset(duel, RESET_PHASE | PHASE_DAMAGE_CAL);
    e1.set_value(duel, 3000);
    c.register_effect(duel, e1);
    Ok(())
}
