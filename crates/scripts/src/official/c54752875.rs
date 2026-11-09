// Twin-Headed Thunder Dragon
use crate::prelude::*;
use crate::testing::{Check, EXTRA, HAND};

const ID: u32 = 54752875;

struct Materials;

impl FusionMix for Materials {
    const SUB: bool = true;
    const INSF: bool = true;
    const MATERIALS: &'static [FusionMaterial] = &[FusionMaterial::Code(31786629); 2];
}

// In the Extra Deck, Fusion Summoned by Polymerization with two Thunder Dragons.
pub const SUPPORT: &[(u32, u16)] = &[(ID, EXTRA), (CARD_POLYMERIZATION, HAND), (31786629, HAND), (31786629, HAND)];
pub const CHECKS: &[Check] = &[Check::SpecialSummoned];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Fusion::add_proc_mix_n::<Materials>(duel, c);
}
