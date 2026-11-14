// Gem-Knight Seraphinite
use crate::prelude::*;
use crate::testing::{Check, EXTRA, HAND};

const ID: u32 = 3113836;

pub const LISTED_SERIES: &[u16] = &[SET_GEM_KNIGHT];
pub const MATERIAL_SETCODE: &[u16] = &[SET_GEM, SET_GEM_KNIGHT];

struct Materials;

impl FusionMix for Materials {
    const SUB: bool = false;
    const INSF: bool = false;
    const MATERIALS: &'static [FusionMaterial] = &[
        FusionMaterial::Filter(gem_knight),
        FusionMaterial::Filter(|c, duel, fc, sumtype, tp| c.is_attribute_full(duel, ATTRIBUTE_LIGHT, Some(fc), sumtype, tp)),
    ];
}

// In the Extra Deck, Fusion Summoned by Gem-Knight Fusion with Gem-Knight Nepyrim (LIGHT) and Garnet from the hand, with more
// Garnet in the hand for the extra Normal Summon.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (1264319, HAND),
    (CARD_POLYMERIZATION, HAND),
    (51831560, HAND),
    (91731841, HAND),
    (91731841, HAND),
];
pub const CHECKS: &[Check] = &[Check::SpecialSummoned];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Fusion::add_proc_mix::<Materials>(duel, c);
    register_splimit(duel, c);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 0));
    e2.set_type(duel, EFFECT_TYPE_FIELD);
    e2.set_target_range(duel, LOCATION_HAND | LOCATION_MZONE, 0);
    e2.set_code(duel, EFFECT_EXTRA_SUMMON_COUNT);
    e2.set_range(duel, LOCATION_MZONE);
    c.register_effect(duel, e2);
    let e3 = e2.clone_effect(duel);
    e3.set_code(duel, EFFECT_EXTRA_SET_COUNT);
    c.register_effect(duel, e3);
}

pub fn gem_knight(c: CardId, duel: &Duel, fc: CardId, sumtype: u64, tp: u8) -> bool {
    c.is_set_card_full(duel, &[SET_GEM_KNIGHT], Some(fc), sumtype, tp)
}

pub fn register_splimit(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE + EFFECT_FLAG_UNCOPYABLE);
    e1.set_code(duel, EFFECT_SPSUMMON_CONDITION);
    e1.set_value(duel, Value::SpSummonCondition(splimit));
    c.register_effect(duel, e1);
}

fn splimit(duel: &Duel, e: EffectId, _: Option<EffectId>, _: u8, st: u32, _: u8, _: u8) -> bool {
    !e.get_handler(duel).is_location(duel, LOCATION_EXTRA) || st & SUMMON_TYPE_FUSION == SUMMON_TYPE_FUSION
}
