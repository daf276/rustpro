// Traffic Ghost
use crate::prelude::*;
use crate::testing::{Check, EXTRA, MZONE};

const ID: u32 = 41248270;

struct Procedure;

impl LinkProcedure for Procedure {
    const MIN: usize = 3;
    const MAX: Option<usize> = Some(3);
}

// In the Extra Deck, Link Summoned with three monsters.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (24154052, MZONE),
    (24154052, MZONE),
    (24154052, MZONE),
    (24154052, MZONE),
    (24154052, MZONE),
];
// No effect.
pub const CHECKS: &[Check] = &[Check::SpecialSummoned];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Link::add_procedure::<Procedure>(duel, c);
}
