// Gladiator Proving Ground
use crate::prelude::*;
use crate::testing::{DECK, HAND};

const ID: u32 = 35224440;

pub const LISTED_SERIES: &[u16] = &[SET_GLADIATOR];

// In the hand, with Gladiator Beasts (Bestiari, Darius) in the Deck to add.
pub const SUPPORT: &[(u32, u16)] = &[(ID, HAND), (41470137, DECK), (41470137, DECK), (25924653, DECK)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_category(duel, CATEGORY_TOHAND + CATEGORY_SEARCH);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_target(duel, target);
    e1.set_operation(duel, activate);
    c.register_effect(duel, e1);
}

fn filter(c: CardId, duel: &Duel) -> bool {
    let lv = c.get_level(duel);
    c.is_set_card(duel, &[SET_GLADIATOR]) && lv > 0 && lv < 5 && c.is_able_to_hand(duel)
}

fn target(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    stock::exists_target(duel, a, chk, filter, LOCATION_DECK, CATEGORY_TOHAND)
}

fn activate(duel: &mut Duel, a: &Args) -> Script<()> {
    stock::search(duel, a, filter, LOCATION_DECK)
}
