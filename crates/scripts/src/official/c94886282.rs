// Charge of the Light Brigade
use crate::prelude::*;
use crate::testing::{DECK, HAND};

const ID: u32 = 94886282;

pub const LISTED_SERIES: &[u16] = &[SET_LIGHTSWORN];
// Copies in the hand, and Lightsworn monsters of Level 4 or lower (Lyla, Wulf, Jain) in the Deck to search.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, HAND),
    (22624373, DECK),
    (22624373, DECK),
    (58996430, DECK),
    (96235275, DECK),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_category(duel, CATEGORY_TOHAND + CATEGORY_SEARCH);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_cost(duel, cost);
    e1.set_target(duel, target);
    e1.set_operation(duel, activate);
    c.register_effect(duel, e1);
}

fn cost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    if chk == 0 {
        return Ok(duel.is_player_can_discard_deck_as_cost(a.tp, 3) && duel.get_field_group_count(a.tp, LOCATION_DECK, 0) > 3);
    }
    duel.discard_deck(a.tp, 3, REASON_COST)?;
    Ok(false)
}

fn filter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_LIGHTSWORN]) && c.is_level_below(duel, 4) && c.is_able_to_hand(duel)
}

fn target(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    stock::exists_target(duel, a, chk, filter, LOCATION_DECK, CATEGORY_TOHAND)
}

fn activate(duel: &mut Duel, a: &Args) -> Script<()> {
    stock::search(duel, a, filter, LOCATION_DECK)
}
