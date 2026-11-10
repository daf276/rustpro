// Deepsea Shark
use crate::prelude::*;
use crate::testing::{Check, EXTRA, HAND};

const ID: u32 = 28593363;
const BOTTOM_DWELLER: u32 = 81386177;
const TONGYO: u32 = 69572024;

struct Materials;

impl FusionMix for Materials {
    const SUB: bool = true;
    const INSF: bool = true;
    const MATERIALS: &'static [FusionMaterial] = &[FusionMaterial::Code(BOTTOM_DWELLER), FusionMaterial::Code(TONGYO)];
}

// In the Extra Deck, Fusion Summoned by Polymerization with its two Normal Monsters in the hand.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (CARD_POLYMERIZATION, HAND),
    (CARD_POLYMERIZATION, HAND),
    (BOTTOM_DWELLER, HAND),
    (BOTTOM_DWELLER, HAND),
    (TONGYO, HAND),
    (TONGYO, HAND),
];
// It has no effect to resolve.
pub const CHECKS: &[Check] = &[Check::SpecialSummoned];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Fusion::add_proc_mix::<Materials>(duel, c);
}
