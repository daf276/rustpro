// Number 4: Numeron Gate Catvari
use crate::prelude::*;
use crate::testing::{EXTRA, MZONE, OPPONENT};

const ID: u32 = 4019153;

pub const LISTED_SERIES: &[u16] = &[SET_NUMERON];
pub const XYZ_NUMBER: u32 = 4;

// In the Extra Deck, Xyz Summoned with Level 1 Normal Monsters, with the opponent's Elfnote Lucina and Ash Blossom to attack into it.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (1929294, MZONE),
    (3557275, MZONE),
    (7902349, MZONE),
    (8124921, MZONE),
    (13597785, MZONE | OPPONENT),
    (14558127, MZONE | OPPONENT),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    super::c15232745::add_effects(duel, c, ID);
}
