// Elemental HERO Mariner
use engine::constants::MSG_ATTACK;

use crate::prelude::*;
use crate::testing::{Check, EXTRA, HAND, MZONE, OPPONENT, SZONE};

const ID: u32 = 14225239;
pub const MATERIAL_SETCODE: &[u16] = &[SET_HERO, SET_ELEMENTAL_HERO];

struct Materials;

impl FusionMix for Materials {
    const SUB: bool = true;
    const INSF: bool = true;
    const MATERIALS: &'static [FusionMaterial] = &[FusionMaterial::Code(79979666), FusionMaterial::Code(21844576)];
}

// In the Extra Deck and on the field, Fusion Summoned by Polymerization with Elemental HERO Bubbleman and Avian, with a Set
// Mirror Force for the direct attack past the opponent's Sangan.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, MZONE),
    (CARD_POLYMERIZATION, HAND),
    (79979666, HAND),
    (21844576, HAND),
    (44095762, SZONE),
    (26202165, MZONE | OPPONENT),
];
// It has no effect to resolve.
pub const CHECKS: &[Check] = &[Check::SpecialSummoned, Check::Message(MSG_ATTACK)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Fusion::add_proc_mix::<Materials>(duel, c);
    c.add_must_be_fusion_summoned(duel);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_SINGLE);
    e2.set_code(duel, EFFECT_DIRECT_ATTACK);
    e2.set_condition(duel, dacon);
    c.register_effect(duel, e2);
}

fn filter(c: CardId, duel: &Duel) -> bool {
    c.is_facedown(duel) && c.get_sequence(duel) != 5
}

fn dacon(duel: &Duel, a: &Args) -> bool {
    duel.is_existing_matching_card(filter, a.e.get_handler_player(duel), LOCATION_SZONE, 0, 1, None)
}
