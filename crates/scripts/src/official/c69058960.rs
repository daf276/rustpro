// Number 13: Embodiment of Crime
use super::c95442074::register;
use crate::prelude::*;
use crate::testing::{MZONE, OPPONENT};

const ID: u32 = 69058960;
const EMBODIMENT_OF_PUNISHMENT: u32 = 95442074;
pub const XYZ_NUMBER: u32 = 13;

pub const LISTED_NAMES: &[u32] = &[EMBODIMENT_OF_PUNISHMENT];

// Xyz Summoned with two Level 1 monsters, with an opponent's monster to change the position of. Neither a second copy nor Number 31:
// Embodiment of Punishment is placed, since two of them in play reach the unported choice among several EFFECT_MUST_ATTACK_MONSTER
// effects.
pub const SUPPORT: &[(u32, u16)] = &[(1929294, MZONE), (3557275, MZONE), (13193642, MZONE), (26202165, MZONE | OPPONENT)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    register::<ID, EMBODIMENT_OF_PUNISHMENT>(duel, c);
}
