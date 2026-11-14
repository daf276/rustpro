// Gravekeeper's Commandant
use crate::prelude::*;
use crate::testing::{Check, DECK, HAND};

const ID: u32 = 17393207;

pub const LISTED_NAMES: &[u32] = &[CARD_NECROVALLEY];
// Necrovalley in the Deck to search.
pub const SUPPORT: &[(u32, u16)] = &[(ID, HAND), (CARD_NECROVALLEY, DECK), (CARD_NECROVALLEY, DECK)];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_TOHAND + CATEGORY_SEARCH);
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_range(duel, LOCATION_HAND);
    e1.set_cost(duel, Cost::self_discard_to_grave);
    e1.set_target(duel, target);
    e1.set_operation(duel, operation);
    c.register_effect(duel, e1);
}

fn filter(c: CardId, duel: &Duel) -> bool {
    c.is_code(duel, CARD_NECROVALLEY) && c.is_able_to_hand(duel)
}

fn target(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    stock::exists_target(duel, a, chk, filter, LOCATION_DECK, CATEGORY_TOHAND)
}

fn operation(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    if let Some(tg) = duel.get_first_matching_card(filter, tp, LOCATION_DECK, 0, None) {
        duel.send_to_hand(tg, PLAYER_NONE, REASON_EFFECT)?;
        duel.confirm_cards(1 - tp, tg)?;
    }
    Ok(())
}
