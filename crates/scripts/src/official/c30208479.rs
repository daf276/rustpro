// Magician of Black Chaos
use crate::prelude::*;

pub const LISTED_NAMES: &[u32] = &[76792184];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
}
