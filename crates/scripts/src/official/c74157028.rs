// Cyber Twin Dragon
use crate::prelude::*;
use crate::testing::{Check, EXTRA, HAND};

struct Materials;

impl FusionMix for Materials {
    const SUB: bool = false;
    const INSF: bool = false;
    const MATERIALS: &'static [FusionMaterial] = &[FusionMaterial::Code(CARD_CYBER_DRAGON); 2];
}

pub const MATERIAL_SETCODE: &[u16] = &[SET_CYBER, SET_CYBER_DRAGON];
// In the Extra Deck, Fusion Summoned by Polymerization with two Cyber Dragons in the hand.
pub const SUPPORT: &[(u32, u16)] = &[
    (74157028, EXTRA),
    (CARD_POLYMERIZATION, HAND),
    (CARD_CYBER_DRAGON, HAND),
    (CARD_CYBER_DRAGON, HAND),
];
pub const CHECKS: &[Check] = &[Check::SpecialSummoned];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Fusion::add_proc_mix_n::<Materials>(duel, c);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_SINGLE);
    e2.set_code(duel, EFFECT_EXTRA_ATTACK);
    e2.set_value(duel, 1);
    c.register_effect(duel, e2);
}
