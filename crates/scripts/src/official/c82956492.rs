// D/D/D Oracle King d'Arc
use engine::constants::MSG_RECOVER;

use crate::prelude::*;
use crate::testing::{Check, EXTRA, HAND, MZONE, SZONE};

const ID: u32 = 82956492;

pub const LISTED_SERIES: &[u16] = &[SET_DD];
pub const MATERIAL_SETCODE: &[u16] = &[SET_DD];

struct Materials;

impl FusionMix for Materials {
    const SUB: bool = true;
    const INSF: bool = true;
    const MATERIALS: &'static [FusionMaterial] =
        &[FusionMaterial::Filter(|c, duel, fc, sumtype, tp| c.is_set_card_full(duel, &[SET_DD], Some(fc), sumtype, tp)); 2];
}

// On the field with Dark Contract with the Witch, whose Standby Phase damage it turns into a recovery; fused by Polymerization
// from D/D Pandora and D/D Orthros.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (ID, EXTRA),
    (9765723, SZONE),
    (9765723, SZONE),
    (CARD_POLYMERIZATION, HAND),
    (32146097, MZONE),
    (72181263, MZONE),
];
// It has no activated effect: the recovery shows the reversed damage.
pub const CHECKS: &[Check] = &[Check::Message(MSG_RECOVER), Check::SpecialSummoned];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    Fusion::add_proc_mix_n::<Materials>(duel, c);
    c.enable_revive_limit(duel);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_FIELD);
    e2.set_code(duel, EFFECT_REVERSE_DAMAGE);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_property(duel, EFFECT_FLAG_PLAYER_TARGET);
    e2.set_target_range(duel, 1, 0);
    e2.set_value(duel, Value::Reason(|_, _, _, r, _| r & REASON_EFFECT != 0));
    c.register_effect(duel, e2);
}
