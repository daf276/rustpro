// Invoked Magellanica
use crate::prelude::*;
use crate::testing::{Check, EXTRA, HAND};

const ALEISTER_THE_INVOKER: u32 = 86120751;

struct Materials;

impl FusionMix for Materials {
    const SUB: bool = true;
    const INSF: bool = true;
    const MATERIALS: &'static [FusionMaterial] = &[
        FusionMaterial::Code(ALEISTER_THE_INVOKER),
        FusionMaterial::Filter(|c, duel, fc, sumtype, tp| c.is_attribute_full(duel, ATTRIBUTE_EARTH, Some(fc), sumtype, tp)),
    ];
}

// No effects, so only its Special Summon is checked: Invocation, Aleister the Invoker and EARTH Tao the Chanter in hand.
pub const SUPPORT: &[(u32, u16)] = &[(48791583, EXTRA), (74063034, HAND), (86120751, HAND), (46247516, HAND)];
pub const CHECKS: &[Check] = &[Check::SpecialSummoned];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Fusion::add_proc_mix::<Materials>(duel, c);
}
