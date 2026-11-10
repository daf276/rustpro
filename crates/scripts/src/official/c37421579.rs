// Charubin the Fire Knight
use crate::prelude::*;
use crate::testing::{Check, EXTRA, HAND};

const ID: u32 = 37421579;

struct Materials;

impl FusionMix for Materials {
    const SUB: bool = true;
    const INSF: bool = true;
    const MATERIALS: &'static [FusionMaterial] = &[FusionMaterial::Code(36121917), FusionMaterial::Code(96851799)];
}

// In the Extra Deck, Fusion Summoned by Polymerization with Monster Egg and Hinotama Soul.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (CARD_POLYMERIZATION, HAND),
    (36121917, HAND),
    (96851799, HAND),
    (36121917, HAND),
    (96851799, HAND),
];
// It has no effect to resolve.
pub const CHECKS: &[Check] = &[Check::SpecialSummoned];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Fusion::add_proc_mix::<Materials>(duel, c);
}
