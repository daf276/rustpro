// Lyna the Light Charmer
use crate::prelude::*;
use crate::testing::{MZONE, OPPONENT};

const ID: u32 = 73318863;

// Face-down on the field, flipped by Elfnote Lucina's attack, with Key Mace and Mokey Mokey to take.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (ID, MZONE),
    (ID, MZONE),
    (13597785, MZONE | OPPONENT),
    (1929294, MZONE | OPPONENT),
    (27288416, MZONE | OPPONENT),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    super::c759393::register::<ATTRIBUTE_LIGHT>(duel, c, ID);
}
