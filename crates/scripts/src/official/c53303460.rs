// Impcantation Candoll
use super::c80701178::register_summon_effects;
use crate::prelude::*;
use crate::testing::{Check, DECK, HAND};

const ID: u32 = 53303460;
const TALISMANDRA: u32 = 80701178;
const INCEPTION: u32 = 86758915;

pub const LISTED_SERIES: &[u16] = &[SET_IMPCANTATION];
pub const LISTED_NAMES: &[u32] = &[ID];
// In the hand with Impcantation Inception (Ritual Spell) to reveal and search, and Impcantation Talismandra in the Deck to
// summon; also in the Deck for the summons of Talismandra and Impcantation Chalislime (Ritual Monster to reveal) in the hand.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (INCEPTION, HAND),
    (INCEPTION, DECK),
    (INCEPTION, DECK),
    (TALISMANDRA, DECK),
    (TALISMANDRA, HAND),
    (TALISMANDRA, HAND),
    (65877963, HAND),
    (ID, DECK),
    (ID, DECK),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    register_summon_effects::<ID, { TYPE_SPELL | TYPE_RITUAL }>(duel, c);
}
