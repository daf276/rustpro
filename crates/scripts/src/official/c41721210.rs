// Dark Magician the Dragon Knight
use crate::prelude::*;
use crate::testing::{Check, EXTRA, HAND};

const ID: u32 = 41721210;

pub const LISTED_NAMES: &[u32] = &[CARD_DARK_MAGICIAN];
pub const MATERIAL_SETCODE: &[u16] = &[SET_DARK_MAGICIAN];

struct Materials;

impl FusionMix for Materials {
    const SUB: bool = true;
    const INSF: bool = true;
    const MATERIALS: &'static [FusionMaterial] = &[
        FusionMaterial::Code(CARD_DARK_MAGICIAN),
        FusionMaterial::Filter(|c, duel, fc, sumtype, tp| c.is_race_full(duel, RACE_DRAGON, Some(fc), sumtype, tp)),
    ];
}

// In the Extra Deck, Fusion Summoned by Polymerization with Dark Magician and Red-Eyes Black Dragon (a Normal Dragon).
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (CARD_POLYMERIZATION, HAND),
    (CARD_DARK_MAGICIAN, HAND),
    (CARD_REDEYES_B_DRAGON, HAND),
];
pub const CHECKS: &[Check] = &[Check::SpecialSummoned];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Fusion::add_proc_mix::<Materials>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_property(duel, EFFECT_FLAG_SINGLE_RANGE);
    e1.set_code(duel, EFFECT_CHANGE_CODE);
    e1.set_range(duel, LOCATION_MZONE | LOCATION_GRAVE);
    e1.set_value(duel, CARD_DARK_MAGICIAN as i32);
    c.register_effect(duel, e1);
    let e3 = duel.create_effect(c);
    e3.set_type(duel, EFFECT_TYPE_FIELD);
    e3.set_code(duel, EFFECT_INDESTRUCTABLE_EFFECT);
    e3.set_property(duel, EFFECT_FLAG_SET_AVAILABLE);
    e3.set_range(duel, LOCATION_MZONE);
    e3.set_target_range(duel, LOCATION_ONFIELD, 0);
    e3.set_target_card(duel, |duel, _, c| c.is_spell_trap(duel));
    e3.set_value(duel, Value::EffectPlayer(aux::indoval));
    c.register_effect(duel, e3);
    let e4 = e3.clone_effect(duel);
    e4.set_code(duel, EFFECT_CANNOT_BE_EFFECT_TARGET);
    e4.set_property(duel, EFFECT_FLAG_SET_AVAILABLE + EFFECT_FLAG_IGNORE_IMMUNE);
    e4.set_value(duel, Value::EffectPlayer(aux::tgoval));
    c.register_effect(duel, e4);
}
