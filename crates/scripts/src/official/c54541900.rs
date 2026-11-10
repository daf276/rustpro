// Karbonala Warrior
use crate::prelude::*;
use crate::testing::{Check, EXTRA, HAND};

struct Materials;

impl FusionMix for Materials {
    const SUB: bool = true;
    const INSF: bool = true;
    const MATERIALS: &'static [FusionMaterial] = &[FusionMaterial::Code(56342351), FusionMaterial::Code(92731455)];
}

// In the Extra Deck, Fusion Summoned by Polymerization with M-Warrior #1 and #2.
pub const SUPPORT: &[(u32, u16)] = &[(54541900, EXTRA), (24094653, HAND), (56342351, HAND), (92731455, HAND)];
pub const CHECKS: &[Check] = &[Check::SpecialSummoned];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Fusion::add_proc_mix::<Materials>(duel, c);
}
