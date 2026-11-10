// Toon Summoned Skull
use engine::constants::MSG_ATTACK;

use crate::prelude::*;
use crate::testing::{Check, HAND, MZONE, OPPONENT, SZONE};

pub const LISTED_NAMES: &[u32] = &[CARD_TOON_WORLD];

// In the hand, Special Summoned by Tributing a monster with Toon World on the field, to attack the opponent's monsters.
pub const SUPPORT: &[(u32, u16)] = &[
    (91842653, HAND),
    (CARD_TOON_WORLD, SZONE),
    (89631139, MZONE),
    (26202165, MZONE | OPPONENT),
];
// It has no effect to resolve.
pub const CHECKS: &[Check] = &[Check::SpecialSummoned, Check::Message(MSG_ATTACK)];

// Same effects as Blue-Eyes Toon Dragon, with one Tribute instead of two.
pub fn initial_effect(duel: &mut Duel, c: CardId) {
    super::c53183600::initial_effect_releasing::<1>(duel, c);
}
