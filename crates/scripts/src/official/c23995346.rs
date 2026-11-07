// Blue-Eyes Ultimate Dragon
use crate::prelude::*;
use crate::testing::{Check, EXTRA, HAND, MZONE};

const ID: u32 = 23995346;
pub const MATERIAL_SETCODE: &[u16] = &[SET_BLUE_EYES];

pub const LISTED_NAMES: &[u32] = &[CARD_BLUEEYES_W_DRAGON];

struct Materials;

impl FusionMix for Materials {
    const SUB: bool = true;
    const INSF: bool = true;
    const MATERIALS: &'static [FusionMaterial] = &[FusionMaterial::Code(CARD_BLUEEYES_W_DRAGON); 3];
}

// In the Extra Deck, Fusion Summoned by Polymerization with three Blue-Eyes White Dragons.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (24094653, HAND),
    (CARD_BLUEEYES_W_DRAGON, MZONE),
    (CARD_BLUEEYES_W_DRAGON, MZONE),
    (CARD_BLUEEYES_W_DRAGON, MZONE),
];
pub const CHECKS: &[Check] = &[Check::SpecialSummoned];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Fusion::add_proc_mix_n::<Materials>(duel, c);
}
