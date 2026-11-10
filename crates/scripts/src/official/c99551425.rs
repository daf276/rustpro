// Labyrinth Tank
use crate::prelude::*;
use crate::testing::{Check, EXTRA, HAND};

struct Materials;

impl FusionMix for Materials {
    const SUB: bool = true;
    const INSF: bool = true;
    const MATERIALS: &'static [FusionMaterial] = &[FusionMaterial::Code(8471389), FusionMaterial::Code(11384280)];
}

// In the Extra Deck, Fusion Summoned by Polymerization with Giga-Tech Wolf and Cannon Soldier.
pub const SUPPORT: &[(u32, u16)] = &[(99551425, EXTRA), (24094653, HAND), (8471389, HAND), (11384280, HAND)];
pub const CHECKS: &[Check] = &[Check::SpecialSummoned];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Fusion::add_proc_mix::<Materials>(duel, c);
}
