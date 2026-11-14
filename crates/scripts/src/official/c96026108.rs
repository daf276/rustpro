// Drytron Zeta Aldhibah
use super::c97148796::{register, spop_search, sptg};
use crate::prelude::*;
use crate::testing::{DECK, HAND, MZONE};

const ID: u32 = 96026108;

pub const LISTED_SERIES: &[u16] = &[SET_DRYTRON];
// Drytron monsters to Tribute, and Meteonis Drytron, a Ritual Spell, to add.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, HAND),
    (97148796, MZONE),
    (60037599, HAND),
    (22398665, DECK),
    (22398665, DECK),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let category = CATEGORY_SPECIAL_SUMMON + CATEGORY_TOHAND + CATEGORY_SEARCH;
    register::<ID>(duel, c, category, sptg::<CATEGORY_TOHAND, { LOCATION_DECK as i32 }>, spop);
}

fn thfilter(c: CardId, duel: &Duel) -> bool {
    c.is_ritual_spell(duel) && c.is_able_to_hand(duel)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    spop_search::<ID>(duel, a, thfilter)
}
