// Sanwitch
use crate::prelude::*;
use crate::testing::{Check, EXTRA, HAND};

struct Materials;

impl FusionMix for Materials {
    const SUB: bool = true;
    const INSF: bool = true;
    const MATERIALS: &'static [FusionMaterial] = &[FusionMaterial::Code(78010363), FusionMaterial::Code(26202165)];
}

// In the Extra Deck, Fusion Summoned by Polymerization with Witch of the Black Forest and Sangan.
pub const SUPPORT: &[(u32, u16)] = &[(53539634, EXTRA), (CARD_POLYMERIZATION, HAND), (78010363, HAND), (26202165, HAND)];
pub const CHECKS: &[Check] = &[Check::SpecialSummoned];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Fusion::add_proc_mix::<Materials>(duel, c);
}
