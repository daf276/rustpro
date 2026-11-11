// Seven Tools of the Bandit
use crate::prelude::*;
use crate::testing::{HAND, OPPONENT, SZONE};

const ID: u32 = 3819470;

// In the hand and Set, with the opponent's Set Mirror Force, Trap Hole and Dimensional Prison to negate.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, SZONE),
    (44095762, SZONE | OPPONENT),
    (4206964, SZONE | OPPONENT),
    (70342110, SZONE | OPPONENT),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_category(duel, CATEGORY_NEGATE + CATEGORY_DESTROY);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_CHAINING);
    e1.set_condition(duel, condition);
    e1.set_cost(duel, Cost::pay_lp::<1000>);
    e1.set_target(duel, stock::negate_destroy_target);
    e1.set_operation(duel, operation);
    c.register_effect(duel, e1);
}

fn condition(duel: &Duel, a: &Args) -> bool {
    a.re.is_some_and(|re| re.is_trap_effect(duel) && re.is_has_type(duel, EFFECT_TYPE_ACTIVATE)) && duel.is_chain_negatable(a.ev as u8)
}

fn operation(duel: &mut Duel, a: &Args) -> Script<()> {
    let Some(re) = a.re else { return Ok(()) };
    if duel.negate_activation(a.ev as u8) && re.get_handler(duel).is_relate_to_effect(duel, re) {
        duel.destroy(a.eg.unwrap_or_default(), REASON_EFFECT)?;
    }
    Ok(())
}
