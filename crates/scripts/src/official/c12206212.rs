// Harpie Lady Sisters
use crate::prelude::*;
use crate::testing::{Check, DECK, HAND, MZONE};

// Special Summoned by Elegant Egotist from the hand or the Deck, with Harpie Lady (a Normal Monster) on the field.
pub const SUPPORT: &[(u32, u16)] = &[
    (90219263, HAND),
    (90219263, HAND),
    (CARD_HARPIE_LADY, MZONE),
    (CARD_HARPIE_LADY, MZONE),
    (12206212, HAND),
    (12206212, DECK),
];
// It has no effect.
pub const CHECKS: &[Check] = &[Check::SpecialSummoned];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
}
