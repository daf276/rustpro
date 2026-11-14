// Blackwing - Pinaki the Waxing Moon
use crate::prelude::*;
use crate::testing::{Check, DECK, HAND, MZONE};

const ID: u32 = 55610595;

pub const LISTED_SERIES: &[u16] = &[SET_BLACKWING];
pub const LISTED_NAMES: &[u32] = &[ID];

// On the field, with Dark Hole in hand to destroy it and Blackwings in the Deck to search.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (ID, MZONE),
    (53129443, HAND),
    (2009101, DECK),
    (2009101, DECK),
    (2009101, DECK),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(0)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_code(duel, EFFECT_CANNOT_BE_SYNCHRO_MATERIAL);
    e1.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE + EFFECT_FLAG_UNCOPYABLE);
    e1.set_value(duel, Value::Card(|duel, _, c| i32::from(!c.is_set_card(duel, &[SET_BLACKWING]))));
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_CONTINUOUS);
    e2.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE);
    e2.set_code(duel, EVENT_TO_GRAVE);
    e2.set_operation(duel, regop);
    c.register_effect(duel, e2);
}

fn regop(duel: &mut Duel, a: &Args) -> Script<()> {
    let c = a.e.get_handler(duel);
    if c.is_previous_location(duel, LOCATION_ONFIELD) {
        let e1 = duel.create_effect(c);
        e1.set_category(duel, CATEGORY_TOHAND + CATEGORY_SEARCH);
        e1.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
        e1.set_code(duel, EVENT_PHASE + PHASE_END);
        e1.set_range(duel, LOCATION_GRAVE);
        e1.set_count_limit(duel, 1, ID);
        e1.set_target(duel, thtg);
        e1.set_operation(duel, thop);
        e1.set_reset(duel, RESETS_STANDARD_PHASE_END);
        c.register_effect(duel, e1);
    }
    Ok(())
}

fn filter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_BLACKWING]) && c.is_monster(duel) && !c.is_code(duel, ID) && c.is_able_to_hand(duel)
}

fn thtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    stock::exists_target(duel, a, chk, filter, LOCATION_DECK, CATEGORY_TOHAND)
}

fn thop(duel: &mut Duel, a: &Args) -> Script<()> {
    stock::search(duel, a, filter, LOCATION_DECK)
}
