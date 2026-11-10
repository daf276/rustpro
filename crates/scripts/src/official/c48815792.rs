// Hiita the Fire Charmer, Ablaze
use crate::prelude::*;
use crate::testing::{Check, DECK, EXTRA, GRAVE, HAND, MZONE, OPPONENT};

const ID: u32 = 48815792;

// Giga-Tech Wolf (FIRE) x4 for the Link Summon, one more in the Deck to add and one in the opponent's Graveyard to Special Summon;
// Raigeki and Dark Hole on the opponent's side destroy it.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (ID, EXTRA),
    (ID, EXTRA),
    (8471389, MZONE),
    (8471389, MZONE),
    (8471389, MZONE),
    (8471389, MZONE),
    (8471389, DECK),
    (8471389, GRAVE | OPPONENT),
    (12580477, HAND | OPPONENT),
    (12580477, HAND | OPPONENT),
    (53129443, HAND | OPPONENT),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0)), Check::ResolvesEffect(0)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    super::c30674956::register::<ID, ATTRIBUTE_FIRE>(duel, c);
}
