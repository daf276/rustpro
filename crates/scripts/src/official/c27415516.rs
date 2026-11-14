// Stream, Dragon Ruler of Droplets
use super::c89185742::add_summon_effect;
use crate::prelude::*;
use crate::testing::{Check, DECK, HAND};

const ID: u32 = 27415516;
const TIDAL: u32 = 26400609;

pub const LISTED_NAMES: &[u32] = &[TIDAL];

// A second copy and Red-Eyes Black Dragon (a Dragon) to discard, with Tidal in the Deck.
pub const SUPPORT: &[(u32, u16)] = &[(ID, HAND), (74677422, HAND), (TIDAL, DECK), (TIDAL, DECK), (TIDAL, DECK)];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    add_summon_effect::<ID, ATTRIBUTE_WATER, TIDAL>(duel, c);
}
