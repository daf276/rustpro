// Reactan, Dragon Ruler of Pebbles
use super::c89185742::add_summon_effect;
use crate::prelude::*;
use crate::testing::{Check, DECK, HAND};

const ID: u32 = 91020571;
const REDOX: u32 = 90411554;

pub const LISTED_NAMES: &[u32] = &[REDOX];

// A second copy and Red-Eyes Black Dragon (a Dragon) to discard, with Redox in the Deck.
pub const SUPPORT: &[(u32, u16)] = &[(ID, HAND), (74677422, HAND), (REDOX, DECK), (REDOX, DECK), (REDOX, DECK)];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    add_summon_effect::<ID, ATTRIBUTE_EARTH, REDOX>(duel, c);
}
