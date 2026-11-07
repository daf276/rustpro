// Black Skull Dragon
use crate::prelude::*;
use crate::testing::{Check, EXTRA, HAND};

const ID: u32 = 11901678;
pub const MATERIAL_SETCODE: &[u16] = &[SET_RED_EYES, SET_ARCHFIEND];

struct Materials;

impl FusionMix for Materials {
    const SUB: bool = true;
    const INSF: bool = true;
    const MATERIALS: &'static [FusionMaterial] = &[
        FusionMaterial::Code(CARD_SUMMONED_SKULL),
        FusionMaterial::Code(CARD_REDEYES_B_DRAGON),
    ];
}

pub const LISTED_NAMES: &[u32] = &[CARD_REDEYES_B_DRAGON];
// In the Extra Deck, Fusion Summoned by Polymerization with Summoned Skull and Red-Eyes Black Dragon (Normal Monsters).
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (CARD_POLYMERIZATION, HAND),
    (CARD_SUMMONED_SKULL, HAND),
    (CARD_REDEYES_B_DRAGON, HAND),
];
// It has no effect.
pub const CHECKS: &[Check] = &[Check::SpecialSummoned];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Fusion::add_proc_mix::<Materials>(duel, c);
}
