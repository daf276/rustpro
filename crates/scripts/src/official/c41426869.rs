// Black Illusion Ritual
use crate::prelude::*;
use crate::testing::{Check, HAND};

pub const FIT_MONSTER: &[u32] = &[64631466];

struct Params;

impl RitualCode for Params {
    const LV: Option<u32> = Some(1);
    const CODES: &'static [u32] = FIT_MONSTER;
}

// Ritual Summons Relinquished, with Level 1 Normal Monsters in the hand as the tribute.
pub const SUPPORT: &[(u32, u16)] = &[(64631466, HAND), (27288416, HAND), (1929294, HAND)];
pub const CHECKS: &[Check] = &[Check::SpecialSummonedCode(64631466)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    Ritual::add_proc_greater_code::<Params>(duel, c);
}
