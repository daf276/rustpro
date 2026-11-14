// Burner, Dragon Ruler of Sparks
use super::c89185742::add_summon_effect;
use crate::prelude::*;
use crate::testing::{Check, DECK, HAND};

const ID: u32 = 53797637;
const BLASTER: u32 = 53804307;

pub const LISTED_NAMES: &[u32] = &[BLASTER];

// A second copy and Red-Eyes Black Dragon (a Dragon) to discard, with Blaster in the Deck.
pub const SUPPORT: &[(u32, u16)] = &[(ID, HAND), (74677422, HAND), (BLASTER, DECK), (BLASTER, DECK), (BLASTER, DECK)];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    add_summon_effect::<ID, ATTRIBUTE_FIRE, BLASTER>(duel, c);
}
