// The Light - Hex-Sealed Fusion
use crate::prelude::*;
use crate::testing::{Check, EXTRA, MZONE};

const ID: u32 = 15717011;
const GILTIA: u32 = 51828629;

// Giltia the D. Knight in the Extra Deck, with Guardian of the Labyrinth beside this card standing in for Protector of the Throne.
pub const SUPPORT: &[(u32, u16)] = &[(ID, MZONE), (GILTIA, EXTRA), (89272878, MZONE)];
pub const CHECKS: &[Check] = &[Check::SpecialSummonedCode(GILTIA)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    super::c52101615::register::<ID, ATTRIBUTE_LIGHT>(duel, c, |duel, a| super::c52101615::operation(duel, a, a.rp));
}
