// Cyber End Dragon
use crate::prelude::*;
use crate::testing::{Check, EXTRA, HAND, MZONE};

const ID: u32 = 1546123;
pub const MATERIAL_SETCODE: &[u16] = &[SET_CYBER, SET_CYBER_DRAGON];

struct Materials;

impl FusionMix for Materials {
    const SUB: bool = false;
    const INSF: bool = false;
    const MATERIALS: &'static [FusionMaterial] = &[FusionMaterial::Code(CARD_CYBER_DRAGON); 3];
}

// In the Extra Deck, Fusion Summoned by Polymerization with three Cyber Dragons.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (24094653, HAND),
    (CARD_CYBER_DRAGON, MZONE),
    (CARD_CYBER_DRAGON, MZONE),
    (CARD_CYBER_DRAGON, MZONE),
];
pub const CHECKS: &[Check] = &[Check::SpecialSummoned];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Fusion::add_proc_mix_n::<Materials>(duel, c);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_SINGLE);
    e2.set_code(duel, EFFECT_PIERCE);
    c.register_effect(duel, e2);
}
