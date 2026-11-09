// Adamancipator Analyzer
use super::c48519867::sptg;
use super::c85914562::{operation, spop, target};
use crate::prelude::*;
use crate::testing::{Check, DECK, HAND, MZONE, OPPONENT};

const ID: u32 = 11302671;

// An opponent's monster (Haniwa) while the field is empty for the hand effect, and Rock Normal Monsters in the Deck.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (84285623, MZONE | OPPONENT),
    (73481154, DECK),
    (73481154, DECK),
    (73481154, DECK),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_range(duel, LOCATION_HAND);
    e1.set_count_limit(duel, 1, ID);
    e1.set_condition(duel, spcon);
    e1.set_target(duel, sptg);
    e1.set_operation(duel, spop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e2.set_type(duel, EFFECT_TYPE_IGNITION);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_target(duel, target);
    e2.set_operation(duel, operation::<ID>);
    c.register_effect(duel, e2);
}

fn spcon(duel: &Duel, a: &Args) -> bool {
    duel.get_field_group_count(a.tp, 0, LOCATION_MZONE) > 0 && duel.get_field_group_count(a.tp, LOCATION_MZONE, 0) == 0
}
