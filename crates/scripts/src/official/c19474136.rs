// Mandragon
use crate::prelude::*;
use crate::testing::HAND;

const ID: u32 = 19474136;

// In the hand to activate as a Pendulum Spell.
pub const SUPPORT: &[(u32, u16)] = &[(ID, HAND), (ID, HAND)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    Pendulum::add_procedure(duel, c);
}
