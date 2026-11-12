// Gem-Knight Pearl
use crate::prelude::*;
use crate::testing::{Check, EXTRA, MZONE};

const ID: u32 = 71594310;

// The Xyz procedure of the monsters here that take any monsters of one Level.
pub struct Procedure<const LV: u32, const CT: usize>;

impl<const LV: u32, const CT: usize> XyzProcedure for Procedure<LV, CT> {
    const LV: Option<u32> = Some(LV);
    const CT: usize = CT;
}

// In the Extra Deck, Xyz Summoned with Level 4 Normal Monsters (Prevent Rat, Kojikocy, Uraby).
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
    Xyz::add_procedure::<Procedure<4, 2>>(duel, c);
    c.enable_revive_limit(duel);
}
