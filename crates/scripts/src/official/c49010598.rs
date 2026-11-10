// Divine Wrath
use crate::prelude::*;
use crate::testing::{Check, HAND, MZONE, OPPONENT};

const ID: u32 = 49010598;

// In the hand with a discard, to negate the opponent's Cyber Dragon Nachster and Mulcharmy Fuwalos.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, HAND),
    (13597785, HAND),
    (1142880, MZONE | OPPONENT),
    (42141493, HAND | OPPONENT),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(0)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_category(duel, CATEGORY_NEGATE + CATEGORY_DESTROY);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_CHAINING);
    e1.set_condition(duel, condition);
    e1.set_cost(duel, super::c77414722::cost);
    e1.set_target(duel, super::c77414722::target);
    e1.set_operation(duel, super::c77414722::activate);
    c.register_effect(duel, e1);
}

fn condition(duel: &Duel, a: &Args) -> bool {
    a.re.is_some_and(|re| re.is_monster_effect(duel)) && duel.is_chain_negatable(a.ev as u8)
}
