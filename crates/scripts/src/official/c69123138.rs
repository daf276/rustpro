// Zera the Mant
use crate::prelude::*;
use crate::testing::{Check, HAND};

pub const LISTED_NAMES: &[u32] = &[81756897];

// Zera Ritual, which Ritual Summons it, with Ritual Raven as the whole tribute.
pub const SUPPORT: &[(u32, u16)] = &[(81756897, HAND), (34334692, HAND)];
pub const CHECKS: &[Check] = &[Check::SpecialSummoned];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
}
