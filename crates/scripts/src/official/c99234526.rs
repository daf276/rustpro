// White Dragon Wyverburster
use super::c61901281;
use crate::prelude::*;
use crate::testing::{Check, DECK, GRAVE, HAND, MZONE, OPPONENT};

const ID: u32 = 99234526;
const BLACK_DRAGON_COLLAPSERPENT: u32 = 61901281;

pub const LISTED_NAMES: &[u32] = &[BLACK_DRAGON_COLLAPSERPENT];

// In the hand with Skull Servant (DARK) in the Graveyard and on the field to banish and Black Dragon Collapserpent in the Deck
// to search, and the opponent's Dark Hole to send it to the Graveyard.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, HAND),
    (32274490, GRAVE),
    (32274490, GRAVE),
    (32274490, MZONE),
    (BLACK_DRAGON_COLLAPSERPENT, DECK),
    (BLACK_DRAGON_COLLAPSERPENT, DECK),
    (53129443, HAND | OPPONENT),
];
pub const CHECKS: &[Check] = &[Check::SpecialSummoned, Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c61901281::register::<ID, ATTRIBUTE_DARK, BLACK_DRAGON_COLLAPSERPENT>(duel, c);
}
