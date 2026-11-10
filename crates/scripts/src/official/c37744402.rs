// Wynn the Wind Charmer
use crate::prelude::*;
use crate::testing::{MZONE, OPPONENT};

const ID: u32 = 37744402;

// Face-down on the field, flipped by Elfnote Lucina's attack, with Dancing Elf and Bat to take.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (ID, MZONE),
    (ID, MZONE),
    (13597785, MZONE | OPPONENT),
    (59983499, MZONE | OPPONENT),
    (72076281, MZONE | OPPONENT),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    super::c759393::register::<ATTRIBUTE_WIND>(duel, c, ID);
}
