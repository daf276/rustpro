// Dragoness the Wicked Knight
use crate::prelude::*;
use crate::testing::{Check, EXTRA, HAND};

struct Materials;

impl FusionMix for Materials {
    const SUB: bool = true;
    const INSF: bool = true;
    const MATERIALS: &'static [FusionMaterial] = &[FusionMaterial::Code(53153481), FusionMaterial::Code(33064647)];
}

// In the Extra Deck, Fusion Summoned by Polymerization with Armaill and One-Eyed Shield Dragon (Normal Monsters).
pub const SUPPORT: &[(u32, u16)] = &[(70681994, EXTRA), (CARD_POLYMERIZATION, HAND), (53153481, HAND), (33064647, HAND)];
pub const CHECKS: &[Check] = &[Check::SpecialSummoned];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Fusion::add_proc_mix::<Materials>(duel, c);
}
