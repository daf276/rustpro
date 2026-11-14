// Volcanic Shell
use crate::prelude::*;
use crate::testing::{DECK, GRAVE};

const ID: u32 = 33365932;

pub const LISTED_NAMES: &[u32] = &[ID];

// In the Graveyard, with copies in the Deck to add.
pub const SUPPORT: &[(u32, u16)] = &[(ID, GRAVE), (ID, DECK), (ID, DECK)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_TOHAND + CATEGORY_SEARCH);
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_range(duel, LOCATION_GRAVE);
    e1.set_count_limit(duel, 1, 0);
    e1.set_cost(duel, Cost::pay_lp::<500>);
    e1.set_target(duel, tg);
    e1.set_operation(duel, op);
    c.register_effect(duel, e1);
}

fn filter(c: CardId, duel: &Duel) -> bool {
    c.is_code(duel, ID) && c.is_able_to_hand(duel)
}

fn tg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    stock::exists_target(duel, a, chk, filter, LOCATION_DECK, CATEGORY_TOHAND)
}

fn op(duel: &mut Duel, a: &Args) -> Script<()> {
    if !a.e.get_handler(duel).is_location(duel, LOCATION_GRAVE) {
        return Ok(());
    }
    if let Some(tc) = duel.get_first_matching_card(filter, a.tp, LOCATION_DECK, 0, None) {
        duel.send_to_hand(tc, PLAYER_NONE, REASON_EFFECT)?;
        duel.confirm_cards(1 - a.tp, tc)?;
    }
    Ok(())
}
