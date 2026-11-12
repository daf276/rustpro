// Gagagigo the Risen
use super::c71594310::Procedure;
use crate::prelude::*;
use crate::testing::{Check, EXTRA, MZONE};

const ID: u32 = 9053187;

// In the Extra Deck, Xyz Summoned with three Level 4 Normal Monsters (Prevent Rat, Kojikocy, Uraby).
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (549481, MZONE),
    (1184620, MZONE),
    (1784619, MZONE),
    (1784619, MZONE),
];
// No effect.
pub const CHECKS: &[Check] = &[Check::SpecialSummoned];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    Xyz::add_procedure::<Procedure<4, 3>>(duel, c);
    c.enable_revive_limit(duel);
}
