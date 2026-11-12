// Naturia Leodrake
use super::c2956282::Procedure;
use crate::prelude::*;
use crate::testing::{Check, EXTRA, MZONE};

const ID: u32 = 16527176;

// In the Extra Deck, Synchro Summoned with Naturia Tulip (a Level 2 Tuner) and Gaia The Fierce Knight (Level 7). It has no effect.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (ID, EXTRA),
    (70261145, MZONE),
    (70261145, MZONE),
    (6368038, MZONE),
    (6368038, MZONE),
];
pub const CHECKS: &[Check] = &[Check::SpecialSummoned];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    Synchro::add_procedure::<Procedure>(duel, c);
    c.enable_revive_limit(duel);
}
