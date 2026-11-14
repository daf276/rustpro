// Floowandereeze & Eglen
use super::c18940725::{cost, register_banish_effects, search_and_summon};
use crate::prelude::*;
use crate::testing::{Check, DECK, HAND, REMOVED};

const ID: u32 = 54334420;

// Copies in hand to Normal Summon and banished to return, Level 8 Winged Beasts in the Deck to add, and Robina in hand to summon.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, HAND),
    (ID, REMOVED),
    (61441708, DECK),
    (66431519, DECK),
    (61441708, DECK),
    (18940725, HAND),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_TOHAND + CATEGORY_SEARCH + CATEGORY_SUMMON);
    e1.set_type(duel, EFFECT_TYPE_TRIGGER_O + EFFECT_TYPE_SINGLE);
    e1.set_property(duel, EFFECT_FLAG_DELAY);
    e1.set_code(duel, EVENT_SUMMON_SUCCESS);
    e1.set_count_limit(duel, 1, ID);
    e1.set_cost(duel, cost::<ID>);
    e1.set_target(duel, nstg);
    e1.set_operation(duel, nsop);
    c.register_effect(duel, e1);
    register_banish_effects::<ID>(duel, c);
}

fn filter(c: CardId, duel: &Duel) -> bool {
    c.is_level_above(duel, 7) && c.is_race(duel, RACE_WINGEDBEAST) && c.is_able_to_hand(duel)
}

fn nstg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    stock::exists_target(duel, a, chk, filter, LOCATION_DECK, CATEGORY_TOHAND)
}

fn nsop(duel: &mut Duel, a: &Args) -> Script<()> {
    search_and_summon::<ID>(duel, a.tp, filter)
}
