// Shell of Chaos
use crate::prelude::*;
use crate::testing::{Check, HAND, REMOVED};

const ID: u32 = 77312273;

pub const LISTED_NAMES: &[u32] = &[3806388];

// Kewl Tune Cue (LIGHT) in the hand to banish as the cost, and Core of Chaos banished to Special Summon.
pub const SUPPORT: &[(u32, u16)] = &[(ID, HAND), (16387555, HAND), (16387555, HAND), (3806388, REMOVED)];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    super::c3806388::initial_effect_for::<ID, ATTRIBUTE_LIGHT, 3806388>(duel, c);
}
