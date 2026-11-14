// Lunalight Panther Dancer
use engine::constants::MSG_ATTACK;

use super::c81196066::LUNALIGHT_MATERIAL;
use crate::prelude::*;
use crate::testing::{Check, MZONE, OPPONENT};

const ID: u32 = 97165977;

pub const LISTED_SERIES: &[u16] = &[SET_LUNALIGHT];

const LUNALIGHT_CAT_DANCER: u32 = 51777272;
pub const MATERIAL_SETCODE: &[u16] = &[SET_LUNALIGHT];

struct Materials;

impl FusionMix for Materials {
    const SUB: bool = false;
    const INSF: bool = false;
    const MATERIALS: &'static [FusionMaterial] = &[FusionMaterial::Code(LUNALIGHT_CAT_DANCER), LUNALIGHT_MATERIAL];
}

// Two on the field to attack the opponent's two Sangan; Lunalight Cat Dancer, its material, is not ported.
pub const SUPPORT: &[(u32, u16)] = &[(ID, MZONE), (ID, MZONE), (26202165, MZONE | OPPONENT), (26202165, MZONE | OPPONENT)];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
    Check::Message(MSG_ATTACK),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Fusion::add_proc_mix::<Materials>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE + EFFECT_FLAG_UNCOPYABLE);
    e1.set_code(duel, EFFECT_SPSUMMON_CONDITION);
    e1.set_value(duel, Value::SpSummonCondition(splimit));
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_SINGLE);
    e2.set_code(duel, EFFECT_INDESTRUCTABLE_EFFECT);
    e2.set_property(duel, EFFECT_FLAG_SINGLE_RANGE);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_value(duel, Value::EffectPlayer(aux::indoval));
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 0));
    e3.set_type(duel, EFFECT_TYPE_IGNITION);
    e3.set_count_limit(duel, 1, 0);
    e3.set_range(duel, LOCATION_MZONE);
    e3.set_condition(duel, |duel, _| duel.is_able_to_enter_bp());
    e3.set_operation(duel, operation);
    c.register_effect(duel, e3);
    let e4 = duel.create_effect(c);
    e4.set_description(duel, aux::stringid(ID, 1));
    e4.set_category(duel, CATEGORY_ATKCHANGE);
    e4.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_F);
    e4.set_code(duel, EVENT_BATTLE_DESTROYING);
    e4.set_condition(duel, aux::bdocon);
    e4.set_operation(duel, atkop);
    c.register_effect(duel, e4);
}

fn splimit(duel: &Duel, e: EffectId, _: Option<EffectId>, _: u8, st: u32, _: u8, _: u8) -> bool {
    !e.get_handler(duel).is_location(duel, LOCATION_EXTRA) || st & SUMMON_TYPE_FUSION == SUMMON_TYPE_FUSION
}

fn operation(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_code(duel, EFFECT_INDESTRUCTABLE_COUNT);
    e1.set_target_range(duel, 0, LOCATION_MZONE);
    e1.set_value(duel, Value::Reason(|_, _, _, r, _| r & REASON_BATTLE != 0));
    e1.set_reset(duel, RESET_PHASE + PHASE_END);
    duel.register_effect(e1, tp);
    if c.is_relate_to_effect(duel, e) {
        let e2 = duel.create_effect(c);
        e2.set_type(duel, EFFECT_TYPE_SINGLE);
        e2.set_code(duel, EFFECT_ATTACK_ALL);
        e2.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE);
        e2.set_reset(duel, RESETS_STANDARD_PHASE_END);
        e2.set_value(duel, 2);
        c.register_effect(duel, e2);
    }
    Ok(())
}

fn atkop(duel: &mut Duel, a: &Args) -> Script<()> {
    let c = a.e.get_handler(duel);
    if c.is_relate_to_effect(duel, a.e) && c.is_faceup(duel) {
        let e1 = duel.create_effect(c);
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_code(duel, EFFECT_UPDATE_ATTACK);
        e1.set_value(duel, 200);
        e1.set_reset(duel, RESET_EVENT + RESETS_STANDARD_DISABLE + RESET_PHASE + PHASE_BATTLE);
        c.register_effect(duel, e1);
    }
    Ok(())
}
