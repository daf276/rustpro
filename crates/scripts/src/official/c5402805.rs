// Skyforce Kishin
use crate::prelude::*;
use crate::testing::{Check, EXTRA, MZONE};

const ID: u32 = 5402805;

struct Procedure;

impl LinkProcedure for Procedure {
    const MIN: usize = 2;
    const MAX: Option<usize> = Some(3);

    fn specialchk(duel: &Duel, sg: CardSet, lc: CardId, sumtype: u64, tp: u8) -> bool {
        sg.is_exists(duel, |c, duel| c.is_type_full(duel, TYPE_LINK, Some(lc), sumtype, tp), 1, None)
    }
}

// In the Extra Deck, Link Summoned from Monk of the Tenyi (Link-1) and Mystery Shell Dragon. It has no effect, so its Link Summon is the check.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (32519092, MZONE),
    (32519092, MZONE),
    (18108166, MZONE),
    (18108166, MZONE),
];
pub const CHECKS: &[Check] = &[Check::SpecialSummoned];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    Link::add_procedure::<Procedure>(duel, c);
    c.enable_revive_limit(duel);
}
