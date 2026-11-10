// Dark Elf
use engine::constants::MSG_PAY_LPCOST;

use crate::prelude::*;
use crate::testing::{Check, MZONE, OPPONENT};

const ID: u32 = 21417692;

// On the field to attack the opponent's Sangan.
pub const SUPPORT: &[(u32, u16)] = &[(ID, MZONE), (ID, MZONE), (26202165, MZONE | OPPONENT)];
// The attack cost is a continuous effect with no chain link, so its LP payment is what is counted.
pub const CHECKS: &[Check] = &[Check::Message(MSG_PAY_LPCOST)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_code(duel, EFFECT_ATTACK_COST);
    e1.set_cost_condition(duel, |duel, _, _, tp, _| duel.check_lp_cost(tp, 1000));
    e1.set_operation(duel, atop);
    c.register_effect(duel, e1);
}

fn atop(duel: &mut Duel, a: &Args) -> Script<()> {
    if duel.is_attack_cost_paid() != 2 && a.e.get_handler(duel).is_location(duel, LOCATION_MZONE) {
        duel.pay_lp_cost(a.tp, 1000)?;
        duel.attack_cost_paid();
    }
    Ok(())
}
