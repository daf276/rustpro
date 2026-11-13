// Bracchio-raidus
use crate::prelude::*;
use crate::testing::{Check, EXTRA, HAND};

struct Materials;

impl FusionMix for Materials {
    const SUB: bool = true;
    const INSF: bool = true;
    const MATERIALS: &'static [FusionMaterial] = &[FusionMaterial::Code(94119974), FusionMaterial::Code(38289717)];
}

// In the Extra Deck, Fusion Summoned by Polymerization with Two-Headed King Rex and Crawling Dragon #2.
pub const SUPPORT: &[(u32, u16)] = &[(16507828, EXTRA), (CARD_POLYMERIZATION, HAND), (94119974, HAND), (38289717, HAND)];
// It has no effect to resolve.
pub const CHECKS: &[Check] = &[Check::SpecialSummoned];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Fusion::add_proc_mix::<Materials>(duel, c);
}
