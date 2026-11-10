// Chimeratech Overdragon
use crate::prelude::*;
use crate::testing::{Check, EXTRA, HAND, MZONE};

const ID: u32 = 64599569;
pub const MATERIAL_SETCODE: &[u16] = &[SET_CYBER, SET_CYBER_DRAGON];

struct Materials;

impl FusionMixRep for Materials {
    const SUB: bool = true;
    const INSF: bool = true;
    const FUN1: FusionMaterial =
        FusionMaterial::Filter(|c, duel, fc, sumtype, tp| c.is_race_full(duel, RACE_MACHINE, Some(fc), sumtype, tp));
    const MINC: i32 = 1;
    const MAXC: i32 = 99;
    const MATERIALS: &'static [FusionMaterial] = &[FusionMaterial::Code(CARD_CYBER_DRAGON)];
}

// In the Extra Deck, Fusion Summoned by Polymerization with Cyber Dragons (Machines) on the field.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (CARD_POLYMERIZATION, HAND),
    (CARD_CYBER_DRAGON, MZONE),
    (CARD_CYBER_DRAGON, MZONE),
    (CARD_CYBER_DRAGON, MZONE),
];
pub const CHECKS: &[Check] = &[Check::SpecialSummoned];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Fusion::add_proc_mix_rep::<Materials>(duel, c);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_SINGLE);
    e2.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE + EFFECT_FLAG_UNCOPYABLE);
    e2.set_code(duel, EFFECT_SPSUMMON_CONDITION);
    e2.set_value(duel, Value::SpSummonCondition(aux::fuslimit));
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_CONTINUOUS);
    e3.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e3.set_operation(duel, sucop);
    c.register_effect(duel, e3);
    let e4 = duel.create_effect(c);
    e4.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_F);
    e4.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e4.set_condition(duel, |duel, a| a.e.get_handler(duel).is_fusion_summoned(duel));
    e4.set_operation(duel, tgop);
    c.register_effect(duel, e4);
}

fn sucop(duel: &mut Duel, a: &Args) -> Script<()> {
    let c = a.e.get_handler(duel);
    let count = c.get_material_count(duel) as i32;
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_code(duel, EFFECT_SET_BASE_ATTACK);
    e1.set_value(duel, count * 800);
    e1.set_reset(duel, RESET_EVENT | RESETS_STANDARD_DISABLE);
    c.register_effect(duel, e1);
    let e2 = e1.clone_effect(duel);
    e2.set_code(duel, EFFECT_SET_BASE_DEFENSE);
    c.register_effect(duel, e2);
    let e3 = e1.clone_effect(duel);
    e3.set_code(duel, EFFECT_EXTRA_ATTACK_MONSTER);
    e3.set_value(duel, count - 1);
    c.register_effect(duel, e3);
    Ok(())
}

fn tgop(duel: &mut Duel, a: &Args) -> Script<()> {
    let g = duel.get_matching_group(aux::TRUE, a.tp, LOCATION_ONFIELD, 0, Some(a.e.get_handler(duel).into()));
    duel.send_to_grave(g, REASON_EFFECT)?;
    Ok(())
}
