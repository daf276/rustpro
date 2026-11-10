// Flame Swordsman
use crate::prelude::*;
use crate::testing::{Check, EXTRA, HAND, MZONE, OPPONENT};
use engine::constants::MSG_ATTACK;

struct Materials;

impl FusionMix for Materials {
    const SUB: bool = true;
    const INSF: bool = true;
    const MATERIALS: &'static [FusionMaterial] = &[FusionMaterial::Code(34460851), FusionMaterial::Code(44287299)];
}

// In the Extra Deck, Fusion Summoned by Polymerization with Flame Manipulator and Masaki the Legendary Swordsman, to attack
// the opponent's Sangan.
pub const SUPPORT: &[(u32, u16)] = &[
    (45231177, EXTRA),
    (45231177, MZONE),
    (CARD_POLYMERIZATION, HAND),
    (34460851, HAND),
    (44287299, HAND),
    (26202165, MZONE | OPPONENT),
    (26202165, MZONE | OPPONENT),
];
// It has no effect to resolve.
pub const CHECKS: &[Check] = &[Check::SpecialSummoned, Check::Message(MSG_ATTACK)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Fusion::add_proc_mix::<Materials>(duel, c);
}
