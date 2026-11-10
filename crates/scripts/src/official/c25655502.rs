// Bickuribox
use crate::prelude::*;
use crate::testing::{Check, EXTRA, HAND};

struct Materials;

impl FusionMix for Materials {
    const SUB: bool = true;
    const INSF: bool = true;
    const MATERIALS: &'static [FusionMaterial] = &[FusionMaterial::Code(93889755), FusionMaterial::Code(13215230)];
}

// In the Extra Deck, Fusion Summoned by Polymerization with Crass Clown and Dream Clown.
pub const SUPPORT: &[(u32, u16)] = &[(25655502, EXTRA), (CARD_POLYMERIZATION, HAND), (93889755, HAND), (13215230, HAND)];
pub const CHECKS: &[Check] = &[Check::SpecialSummoned];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Fusion::add_proc_mix::<Materials>(duel, c);
}
