// Elemental HERO Escuridao
use engine::constants::MSG_ATTACK;

use crate::prelude::*;
use crate::testing::{Check, EXTRA, GRAVE, HAND, MZONE, OPPONENT};

const ID: u32 = 33574806;
pub const MATERIAL_SETCODE: &[u16] = &[SET_HERO, SET_ELEMENTAL_HERO];

struct Materials;

impl FusionMix for Materials {
    const SUB: bool = true;
    const INSF: bool = true;
    const MATERIALS: &'static [FusionMaterial] = &[
        FusionMaterial::Filter(|c, duel, fc, sumtype, tp| c.is_set_card_full(duel, &[SET_ELEMENTAL_HERO], Some(fc), sumtype, tp)),
        FusionMaterial::Filter(|c, duel, fc, sumtype, tp| c.is_attribute_full(duel, ATTRIBUTE_DARK, Some(fc), sumtype, tp)),
    ];
}

// In the Extra Deck and on the field, Fusion Summoned by Polymerization with Elemental HERO Wildheart and the DARK Sangan,
// with Elemental HERO Bladedge in the Graveyard for the ATK.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, MZONE),
    (CARD_POLYMERIZATION, HAND),
    (86188410, HAND),
    (26202165, HAND),
    (59793705, GRAVE),
    (26202165, MZONE | OPPONENT),
];
// It has no effect to resolve.
pub const CHECKS: &[Check] = &[Check::SpecialSummoned, Check::Message(MSG_ATTACK)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Fusion::add_proc_mix::<Materials>(duel, c);
    c.add_must_be_fusion_summoned(duel);
    let e3 = duel.create_effect(c);
    e3.set_type(duel, EFFECT_TYPE_SINGLE);
    e3.set_property(duel, EFFECT_FLAG_SINGLE_RANGE);
    e3.set_range(duel, LOCATION_MZONE);
    e3.set_code(duel, EFFECT_UPDATE_ATTACK);
    e3.set_value(duel, Value::Card(atkup));
    c.register_effect(duel, e3);
}

fn atkup(duel: &Duel, _: EffectId, c: CardId) -> i32 {
    let ct = duel.get_matching_group_count(
        |c, duel| c.is_set_card(duel, &[SET_ELEMENTAL_HERO]),
        c.get_controler(duel),
        LOCATION_GRAVE,
        0,
        None,
    );
    ct as i32 * 100
}
