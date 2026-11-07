// Fusionist
use crate::prelude::*;
use crate::testing::{Check, EXTRA, HAND};

struct Materials;

impl FusionMix for Materials {
    const SUB: bool = true;
    const INSF: bool = true;
    const MATERIALS: &'static [FusionMaterial] = &[FusionMaterial::Code(38142739), FusionMaterial::Code(83464209)];
}

// Fusionist in the Extra Deck with its materials, Petit Angel and Mystical Sheep #2, and Polymerization.
pub const SUPPORT: &[(u32, u16)] = &[(1641882, EXTRA), (24094653, HAND), (38142739, HAND), (83464209, HAND)];
pub const CHECKS: &[Check] = &[Check::SpecialSummoned];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Fusion::add_proc_mix::<Materials>(duel, c);
}
