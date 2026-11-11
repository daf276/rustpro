// Monk of the Tenyi
use crate::prelude::*;
use crate::testing::{Check, EXTRA, HAND, MZONE};

struct Procedure;

impl LinkProcedure for Procedure {
    const MIN: usize = 1;
    const MAX: Option<usize> = Some(1);

    fn f(c: CardId, duel: &Duel, lc: CardId, sumtype: u64, tp: u8) -> bool {
        c.is_set_card_full(duel, &[SET_TENYI], Some(lc), sumtype, tp) && !c.is_type_full(duel, TYPE_LINK, Some(lc), sumtype, tp)
    }
}

// Copies in the Extra Deck to Link Summon with Tenyi Spirit - Adhara and Suruya on the field and in the hand.
pub const SUPPORT: &[(u32, u16)] = &[
    (32519092, EXTRA),
    (32519092, EXTRA),
    (98159737, MZONE),
    (52854600, MZONE),
    (98159737, HAND),
    (52854600, HAND),
];
pub const CHECKS: &[Check] = &[Check::SpecialSummoned];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Link::add_procedure::<Procedure>(duel, c);
}
