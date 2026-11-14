// Mathmech Subtraction
use super::c80965043::add_summon_effect;
use crate::prelude::*;
use crate::testing::{Check, HAND, MZONE};

const ID: u32 = 16360142;

// Copies in the hand, with Dark Magicians on the field to target.
pub const SUPPORT: &[(u32, u16)] = &[(ID, HAND), (ID, HAND), (46986414, MZONE), (46986414, MZONE)];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    add_summon_effect::<ID, true>(duel, c);
}
