// Zera Ritual
use crate::prelude::*;
use crate::testing::HAND;

pub const FIT_MONSTER: &[u32] = &[69123138];

struct Params;

impl RitualCode for Params {
    const LV: Option<u32> = Some(8);
    const CODES: &'static [u32] = FIT_MONSTER;
}

// Ritual Summons Zera the Mant, with Ritual Raven as the whole tribute.
pub const SUPPORT: &[(u32, u16)] = &[(69123138, HAND), (34334692, HAND)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    Ritual::add_proc_greater_code::<Params>(duel, c);
}
