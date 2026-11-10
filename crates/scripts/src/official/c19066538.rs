// Roaring Ocean Snake
use crate::prelude::*;
use crate::testing::{Check, EXTRA, HAND};

const ID: u32 = 19066538;
const MYSTIC_LAMP: u32 = 98049915;
const HYOSUBE: u32 = 2118022;

struct Materials;

impl FusionMix for Materials {
    const SUB: bool = true;
    const INSF: bool = true;
    const MATERIALS: &'static [FusionMaterial] = &[FusionMaterial::Code(MYSTIC_LAMP), FusionMaterial::Code(HYOSUBE)];
}

// In the Extra Deck, Fusion Summoned by Polymerization with Mystic Lamp and Hyosube, a Normal Monster, in the hand.
pub const SUPPORT: &[(u32, u16)] = &[(ID, EXTRA), (24094653, HAND), (MYSTIC_LAMP, HAND), (HYOSUBE, HAND)];
pub const CHECKS: &[Check] = &[Check::SpecialSummoned];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Fusion::add_proc_mix::<Materials>(duel, c);
}
