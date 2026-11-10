// Great Mammoth of Goldfine
use crate::prelude::*;
use crate::testing::{Check, EXTRA, HAND, MZONE};

const ID: u32 = 54622031;

struct Materials;

impl FusionMix for Materials {
    const SUB: bool = true;
    const INSF: bool = true;
    const MATERIALS: &'static [FusionMaterial] = &[FusionMaterial::Code(29491031), FusionMaterial::Code(66672569)];
}

// In the Extra Deck, Fusion Summoned by Polymerization with The Snake Hair and Dragon Zombie (Normal Monsters).
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (CARD_POLYMERIZATION, HAND),
    (CARD_POLYMERIZATION, HAND),
    (29491031, MZONE),
    (29491031, HAND),
    (66672569, MZONE),
    (66672569, HAND),
];
// A Fusion Monster with no effect, so only its summon is checked.
pub const CHECKS: &[Check] = &[Check::SpecialSummoned];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Fusion::add_proc_mix::<Materials>(duel, c);
}
