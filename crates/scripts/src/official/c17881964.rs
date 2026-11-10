// Darkfire Dragon
use crate::prelude::*;
use crate::testing::{Check, EXTRA, HAND};

struct Materials;

impl FusionMix for Materials {
    const SUB: bool = true;
    const INSF: bool = true;
    const MATERIALS: &'static [FusionMaterial] = &[FusionMaterial::Code(53293545), FusionMaterial::Code(75356564)];
}

const ID: u32 = 17881964;

// Fusion Summoned by Polymerization with Firegrass and Petit Dragon.
pub const SUPPORT: &[(u32, u16)] = &[(ID, EXTRA), (CARD_POLYMERIZATION, HAND), (53293545, HAND), (75356564, HAND)];
// It has no effect to resolve.
pub const CHECKS: &[Check] = &[Check::SpecialSummoned];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Fusion::add_proc_mix::<Materials>(duel, c);
}
