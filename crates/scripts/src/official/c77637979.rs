// LANphorhynchus
use crate::prelude::*;
use crate::testing::{Check, EXTRA, MZONE};

const ID: u32 = 77637979;

struct Procedure;

impl LinkProcedure for Procedure {
    const MIN: usize = 2;
    const MAX: Option<usize> = Some(2);
}

// No effects: Link Summoned from copies in the Extra Deck with Normal Monsters on the field.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (32295838, MZONE),
    (36211150, MZONE),
    (32295838, MZONE),
    (36211150, MZONE),
];
pub const CHECKS: &[Check] = &[Check::SpecialSummoned];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Link::add_procedure::<Procedure>(duel, c);
}
