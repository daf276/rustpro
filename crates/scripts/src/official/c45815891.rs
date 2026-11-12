// Scrap Archfiend
use crate::prelude::*;
use crate::testing::{Check, EXTRA, MZONE};

const ID: u32 = 45815891;

struct Procedure;

impl SynchroProcedure for Procedure {
    const MIN2: usize = 1;
    const MAX2: usize = 99;

    fn f2(c: CardId, duel: &Duel, sc: CardId, _: u64, tp: u8) -> bool {
        Synchro::non_tuner(c, duel, sc, tp, aux::TRUE)
    }
}

// In the Extra Deck, Synchro Summoned with Scrap Soldier (a Level 5 Tuner) and Mushroom Man (Level 2).
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (ID, EXTRA),
    (65503206, MZONE),
    (65503206, MZONE),
    (14181608, MZONE),
    (14181608, MZONE),
];
// No effect to resolve.
pub const CHECKS: &[Check] = &[Check::SpecialSummoned];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    Synchro::add_procedure::<Procedure>(duel, c);
    c.enable_revive_limit(duel);
}
