// Elemental HERO Rampart Blaster
use engine::constants::MSG_ATTACK;

use crate::prelude::*;
use crate::testing::{Check, EXTRA, HAND, MZONE};

const ID: u32 = 47737087;
pub const MATERIAL_SETCODE: &[u16] = &[SET_HERO, SET_ELEMENTAL_HERO];

struct Materials;

impl FusionMix for Materials {
    const SUB: bool = true;
    const INSF: bool = true;
    const MATERIALS: &'static [FusionMaterial] = &[FusionMaterial::Code(58932615), FusionMaterial::Code(84327329)];
}

// In the Extra Deck and on the field, Fusion Summoned by Polymerization with Elemental HERO Burstinatrix and Clayman, facing
// an empty field to attack directly.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, MZONE),
    (CARD_POLYMERIZATION, HAND),
    (58932615, HAND),
    (84327329, HAND),
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
    let e3 = duel.create_effect(c);
    e3.set_type(duel, EFFECT_TYPE_SINGLE);
    e3.set_code(duel, EFFECT_DEFENSE_ATTACK);
    c.register_effect(duel, e3);
    let e4 = duel.create_effect(c);
    e4.set_type(duel, EFFECT_TYPE_SINGLE);
    e4.set_code(duel, EFFECT_CANNOT_SELECT_BATTLE_TARGET);
    e4.set_condition(duel, dacon);
    e4.set_value(duel, 1);
    c.register_effect(duel, e4);
    let e5 = duel.create_effect(c);
    e5.set_type(duel, EFFECT_TYPE_SINGLE);
    e5.set_code(duel, EFFECT_SET_ATTACK_FINAL);
    e5.set_property(duel, EFFECT_FLAG_SINGLE_RANGE);
    e5.set_range(duel, LOCATION_MZONE);
    e5.set_condition(duel, atkcon);
    e5.set_value(duel, Value::Card(atkval));
    c.register_effect(duel, e5);
}

fn dacon(duel: &Duel, a: &Args) -> bool {
    a.e.get_handler(duel).is_defense_pos(duel)
}

fn atkcon(duel: &Duel, a: &Args) -> bool {
    if duel.get_current_phase() != PHASE_DAMAGE_CAL {
        return false;
    }
    let c = a.e.get_handler(duel);
    c.is_defense_pos(duel)
        && duel.get_attacker() == Some(c)
        && duel.get_attack_target().is_none()
        && c.get_effect_count(duel, EFFECT_DIRECT_ATTACK) == 1
}

fn atkval(duel: &Duel, _: EffectId, c: CardId) -> i32 {
    (c.get_attack(duel) + 1) / 2
}
