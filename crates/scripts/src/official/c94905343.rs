// Rabid Horseman
use crate::prelude::*;
use crate::testing::{Check, EXTRA, HAND};

struct Materials;

impl FusionMix for Materials {
    const SUB: bool = true;
    const INSF: bool = true;
    const MATERIALS: &'static [FusionMaterial] = &[FusionMaterial::Code(5053103), FusionMaterial::Code(68516705)];
}

// In the Extra Deck, Fusion Summoned by Polymerization with Battle Ox and Mystic Horseman.
pub const SUPPORT: &[(u32, u16)] = &[(94905343, EXTRA), (CARD_POLYMERIZATION, HAND), (5053103, HAND), (68516705, HAND)];
pub const CHECKS: &[Check] = &[Check::SpecialSummoned];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Fusion::add_proc_mix::<Materials>(duel, c);
}
