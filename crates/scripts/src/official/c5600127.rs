// Humanoid Worm Drake
use crate::prelude::*;
use crate::testing::{Check, EXTRA, HAND};

struct Materials;

impl FusionMix for Materials {
    const SUB: bool = true;
    const INSF: bool = true;
    const MATERIALS: &'static [FusionMaterial] = &[FusionMaterial::Code(73216412), FusionMaterial::Code(46821314)];
}

// In the Extra Deck, Fusion Summoned by Polymerization with Worm Drake and Humanoid Slime.
pub const SUPPORT: &[(u32, u16)] = &[(5600127, EXTRA), (CARD_POLYMERIZATION, HAND), (73216412, HAND), (46821314, HAND)];
// It has no effect to resolve.
pub const CHECKS: &[Check] = &[Check::SpecialSummoned];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Fusion::add_proc_mix::<Materials>(duel, c);
}
