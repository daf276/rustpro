// Lyna the Light Charmer, Lustrous
use crate::prelude::*;
use crate::testing::{Check, DECK, EXTRA, GRAVE, HAND, MZONE, OPPONENT};

const ID: u32 = 9839945;

// Moon Envoy (LIGHT) x4 for the Link Summon, one more in the Deck to add and one in the opponent's Graveyard to Special Summon;
// Raigeki and Dark Hole on the opponent's side destroy it.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (ID, EXTRA),
    (ID, EXTRA),
    (45909477, MZONE),
    (45909477, MZONE),
    (45909477, MZONE),
    (45909477, MZONE),
    (45909477, DECK),
    (45909477, GRAVE | OPPONENT),
    (12580477, HAND | OPPONENT),
    (12580477, HAND | OPPONENT),
    (53129443, HAND | OPPONENT),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0)), Check::ResolvesEffect(0)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    super::c30674956::register::<ID, ATTRIBUTE_LIGHT>(duel, c);
}
