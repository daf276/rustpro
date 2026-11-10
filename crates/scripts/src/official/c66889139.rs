// Gaia the Dragon Champion
use crate::prelude::*;
use crate::testing::{Check, EXTRA, HAND};

const ID: u32 = 66889139;
const GAIA_THE_FIERCE_KNIGHT: u32 = 6368038;
const CURSE_OF_DRAGON: u32 = 28279543;
pub const MATERIAL_SETCODE: &[u16] = &[SET_GAIA_THE_FIERCE_KNIGHT];

struct Materials;

impl FusionMix for Materials {
    const SUB: bool = true;
    const INSF: bool = true;
    const MATERIALS: &'static [FusionMaterial] = &[FusionMaterial::Code(GAIA_THE_FIERCE_KNIGHT), FusionMaterial::Code(CURSE_OF_DRAGON)];
}

// In the Extra Deck, Fusion Summoned by Polymerization with Gaia The Fierce Knight and Curse of Dragon (Normal Monsters).
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (CARD_POLYMERIZATION, HAND),
    (GAIA_THE_FIERCE_KNIGHT, HAND),
    (CURSE_OF_DRAGON, HAND),
];
// It has no effect.
pub const CHECKS: &[Check] = &[Check::SpecialSummoned];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Fusion::add_proc_mix::<Materials>(duel, c);
}
