// Toll
use crate::prelude::*;
use crate::testing::{Check, HAND, MZONE, OPPONENT, SZONE};
use engine::constants::MSG_PAY_LPCOST;

const ID: u32 = 82003859;

// In the hand and on the field, against an opponent's strong attacker (Lucina) and my monsters to attack.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, SZONE),
    (13597785, MZONE | OPPONENT),
    (13597785, MZONE | OPPONENT),
    (26202165, MZONE),
    (26202165, MZONE),
];
// The attack cost is a continuous effect with no chain link, and its payment has no card.
pub const CHECKS: &[Check] = &[Check::Message(MSG_PAY_LPCOST)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_FIELD);
    e2.set_code(duel, EFFECT_ATTACK_COST);
    e2.set_range(duel, LOCATION_SZONE);
    e2.set_property(duel, EFFECT_FLAG_PLAYER_TARGET);
    e2.set_target_range(duel, 1, 1);
    e2.set_cost_condition(duel, |duel, _, _, tp, _| {
        duel.check_lp_cost(tp, duel.get_player_effect(tp, ID).len() as u32 * 500)
    });
    e2.set_operation(duel, atop);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_type(duel, EFFECT_TYPE_FIELD);
    e3.set_code(duel, ID);
    e3.set_range(duel, LOCATION_SZONE);
    e3.set_property(duel, EFFECT_FLAG_PLAYER_TARGET);
    e3.set_target_range(duel, 1, 1);
    c.register_effect(duel, e3);
}

fn atop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    if duel.is_attack_cost_paid() != 2 && a.e.get_handler(duel).is_location(duel, LOCATION_SZONE) {
        duel.pay_lp_cost(tp, 500)?;
        duel.attack_cost_paid();
    }
    Ok(())
}
