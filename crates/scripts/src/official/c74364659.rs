// Eria the Water Charmer
use crate::prelude::*;
use crate::testing::{MZONE, OPPONENT};

const ID: u32 = 74364659;

// Face-down on the field, flipped by Elfnote Lucina's attack, with Water Spirit and Change Slime to take.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (ID, MZONE),
    (ID, MZONE),
    (13597785, MZONE | OPPONENT),
    (487395, MZONE | OPPONENT),
    (18914778, MZONE | OPPONENT),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    super::c759393::register::<ATTRIBUTE_WATER>(duel, c, ID);
}
