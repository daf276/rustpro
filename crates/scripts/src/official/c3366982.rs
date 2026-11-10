// Alligator's Sword Dragon
use engine::constants::MSG_ATTACK;

use crate::prelude::*;
use crate::testing::{Check, EXTRA, HAND, MZONE, OPPONENT};

const ID: u32 = 3366982;

struct Materials;

impl FusionMix for Materials {
    const SUB: bool = true;
    const INSF: bool = true;
    const MATERIALS: &'static [FusionMaterial] = &[FusionMaterial::Code(88819587), FusionMaterial::Code(64428736)];
}

// Fusion Summoned by Polymerization with Baby Dragon and Alligator's Sword, with another Alligator's Sword (EARTH) on the
// opponent's side to attack directly past.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, MZONE),
    (CARD_POLYMERIZATION, HAND),
    (88819587, HAND),
    (64428736, HAND),
    (64428736, MZONE | OPPONENT),
];
// It has no effect to resolve.
pub const CHECKS: &[Check] = &[Check::SpecialSummoned, Check::Message(MSG_ATTACK)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Fusion::add_proc_mix::<Materials>(duel, c);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_SINGLE);
    e2.set_code(duel, EFFECT_DIRECT_ATTACK);
    e2.set_condition(duel, dircon);
    c.register_effect(duel, e2);
}

fn dircon(duel: &Duel, a: &Args) -> bool {
    let attributes = ATTRIBUTE_EARTH | ATTRIBUTE_WATER | ATTRIBUTE_FIRE;
    let g = duel.get_matching_group(CardId::is_faceup, a.e.get_handler_player(duel), 0, LOCATION_MZONE, None);
    !g.is_empty() && g.filter_count(duel, |c, duel| c.is_attribute(duel, attributes), None) == g.len()
}
