// Aussa the Earth Charmer
use crate::prelude::*;
use crate::testing::{MZONE, OPPONENT};

const ID: u32 = 37970940;

// Face-down on the field, flipped by Elfnote Lucina's attack, with Protron and Petit Moth to take.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (ID, MZONE),
    (ID, MZONE),
    (13597785, MZONE | OPPONENT),
    (24154052, MZONE | OPPONENT),
    (58192742, MZONE | OPPONENT),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    super::c759393::register::<ATTRIBUTE_EARTH>(duel, c, ID);
}
