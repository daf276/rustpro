// Dharc the Dark Charmer
use crate::prelude::*;
use crate::testing::{MZONE, OPPONENT};

const ID: u32 = 19327348;

// Face-down on the field, flipped by Elfnote Lucina's attack, with Skull Servants and Dark Plant to take.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (ID, MZONE),
    (ID, MZONE),
    (13597785, MZONE | OPPONENT),
    (32274490, MZONE | OPPONENT),
    (13193642, MZONE | OPPONENT),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    super::c759393::register::<ATTRIBUTE_DARK>(duel, c, ID);
}
