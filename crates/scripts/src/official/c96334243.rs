// Sea Monster of Theseus
use crate::prelude::*;
use crate::testing::{Check, EXTRA, HAND};

struct Materials;

impl FusionMix for Materials {
    const SUB: bool = true;
    const INSF: bool = true;
    const MATERIALS: &'static [FusionMaterial] =
        &[FusionMaterial::Filter(|c, duel, fc, sumtype, tp| c.is_type_full(duel, TYPE_TUNER, Some(fc), sumtype, tp)); 2];
}

// In the Extra Deck, Fusion Summoned by Polymerization.
pub const SUPPORT: &[(u32, u16)] = &[(96334243, EXTRA), (24094653, HAND)];
pub const CHECKS: &[Check] = &[Check::SpecialSummoned];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Fusion::add_proc_mix_n::<Materials>(duel, c);
}
