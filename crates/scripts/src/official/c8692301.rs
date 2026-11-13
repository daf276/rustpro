// Gem-Knight Zirconia
use crate::prelude::*;
use crate::testing::{Check, EXTRA, HAND};

const ID: u32 = 8692301;

pub const MATERIAL_SETCODE: &[u16] = &[SET_GEM, SET_GEM_KNIGHT];

struct Materials;

impl FusionMix for Materials {
    const SUB: bool = true;
    const INSF: bool = true;
    const MATERIALS: &'static [FusionMaterial] = &[
        FusionMaterial::Filter(|c, duel, fc, sumtype, tp| c.is_set_card_full(duel, &[SET_GEM_KNIGHT], Some(fc), sumtype, tp)),
        FusionMaterial::Filter(|c, duel, fc, sumtype, tp| c.is_race_full(duel, RACE_ROCK, Some(fc), sumtype, tp)),
    ];
}

// In the Extra Deck, Fusion Summoned by Polymerization with Gem-Knight Garnet and Lapis (Rock) from the hand.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (CARD_POLYMERIZATION, HAND),
    (CARD_POLYMERIZATION, HAND),
    (91731841, HAND),
    (91731841, HAND),
    (99645428, HAND),
    (99645428, HAND),
];
pub const CHECKS: &[Check] = &[Check::SpecialSummoned];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Fusion::add_proc_mix::<Materials>(duel, c);
}
