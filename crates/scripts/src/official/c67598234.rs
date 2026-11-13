// Gaia Saber, the Lightning Shadow
use crate::prelude::*;
use crate::testing::{Check, EXTRA, MZONE};

struct Procedure;

impl LinkProcedure for Procedure {
    const MIN: usize = 2;
}

// No effects: Link Summoned from copies in the Extra Deck with Normal Monsters on the field.
pub const SUPPORT: &[(u32, u16)] = &[
    (67598234, EXTRA),
    (67598234, EXTRA),
    (32295838, MZONE),
    (36211150, MZONE),
    (32295838, MZONE),
    (36211150, MZONE),
];
pub const CHECKS: &[Check] = &[Check::SpecialSummoned];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    Link::add_procedure::<Procedure>(duel, c);
    c.enable_revive_limit(duel);
}
