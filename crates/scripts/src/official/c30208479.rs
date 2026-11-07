// Magician of Black Chaos
use crate::prelude::*;
use crate::testing::{Check, HAND};

pub const LISTED_NAMES: &[u32] = &[76792184];

// Black Magic Ritual, which Ritual Summons it.
pub const SUPPORT: &[(u32, u16)] = &[(76792184, HAND)];
pub const CHECKS: &[Check] = &[Check::SpecialSummoned];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
}
