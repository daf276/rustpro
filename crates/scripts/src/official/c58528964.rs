// Flame Ghost
use crate::prelude::*;
use crate::testing::{Check, EXTRA, HAND, MZONE};

const ID: u32 = 58528964;
const DISSOLVEROCK: u32 = 40826495;

struct Materials;

impl FusionMix for Materials {
    const SUB: bool = true;
    const INSF: bool = true;
    const MATERIALS: &'static [FusionMaterial] = &[FusionMaterial::Code(CARD_SKULL_SERVANT), FusionMaterial::Code(DISSOLVEROCK)];
}

pub const LISTED_NAMES: &[u32] = &[CARD_SKULL_SERVANT];
// In the Extra Deck, Fusion Summoned by Polymerization with Skull Servant and Dissolverock.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (CARD_POLYMERIZATION, HAND),
    (CARD_SKULL_SERVANT, HAND),
    (CARD_SKULL_SERVANT, MZONE),
    (DISSOLVEROCK, HAND),
    (DISSOLVEROCK, MZONE),
];
pub const CHECKS: &[Check] = &[Check::SpecialSummoned];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Fusion::add_proc_mix::<Materials>(duel, c);
}
