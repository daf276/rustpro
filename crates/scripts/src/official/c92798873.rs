// Dinomorphia Rexterm
use crate::prelude::*;
use crate::testing::{Check, EXTRA, GRAVE, HAND, MZONE, OPPONENT};

const ID: u32 = 92798873;

pub const LISTED_SERIES: &[u16] = &[SET_DINOMORPHIA];
pub const MATERIAL_SETCODE: &[u16] = &[SET_DINOMORPHIA];

struct Materials;

impl FusionMix for Materials {
    const SUB: bool = true;
    const INSF: bool = true;
    const MATERIALS: &'static [FusionMaterial] = &[
        FusionMaterial::Filter(matfilter),
        FusionMaterial::Filter(|c, duel, fc, sumtype, tp| c.is_set_card_full(duel, &[SET_DINOMORPHIA], Some(fc), sumtype, tp)),
    ];
}

// Fusion Summoned by Polymerization with Kentregina and Therizia; on the field against the opponent's monsters, with Therizia
// in the GY to summon once the opponent's Raigeki destroys it.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, MZONE),
    (24094653, HAND),
    (48832775, MZONE),
    (92133240, HAND),
    (92133240, GRAVE),
    (92133240, GRAVE),
    (26202165, MZONE | OPPONENT),
    (12580477, HAND | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::SpecialSummoned,
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Fusion::add_proc_mix::<Materials>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_code(duel, EFFECT_CANNOT_ACTIVATE);
    e1.set_property(duel, EFFECT_FLAG_PLAYER_TARGET);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_target_range(duel, 0, 1);
    e1.set_value(duel, Value::EffectPlayer(aclimit));
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 0));
    e2.set_category(duel, CATEGORY_ATKCHANGE);
    e2.set_property(duel, EFFECT_FLAG_DAMAGE_STEP);
    e2.set_type(duel, EFFECT_TYPE_QUICK_O);
    e2.set_code(duel, EVENT_FREE_CHAIN);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_hint_timing(
        duel,
        TIMING_DAMAGE_STEP,
        TIMING_DAMAGE_STEP | TIMING_MAIN_END | TIMINGS_CHECK_MONSTER,
    );
    e2.set_count_limit(duel, 1, ID);
    e2.set_condition(duel, aux::stat_change_damage_step_condition);
    e2.set_cost(duel, Cost::pay_lp_half);
    e2.set_target(duel, atktg);
    e2.set_operation(duel, atkop);
    c.register_effect(duel, e2);
    super::c92133240::register_destroyed_summon::<ID, 6, false>(duel, c);
}

fn matfilter(c: CardId, duel: &Duel, fc: CardId, sumtype: u64, tp: u8) -> bool {
    c.is_type_full(duel, TYPE_FUSION, Some(fc), sumtype, tp) && c.is_set_card_full(duel, &[SET_DINOMORPHIA], Some(fc), sumtype, tp)
}

fn aclimit(duel: &Duel, e: EffectId, re: EffectId, _: u8) -> bool {
    let rc = re.get_handler(duel);
    re.is_monster_effect(duel) && rc.is_location(duel, LOCATION_MZONE) && rc.is_attack_above(duel, duel.get_lp(e.get_handler_player(duel)))
}

fn atkfilter(c: CardId, duel: &Duel, atk: i32) -> bool {
    c.is_faceup(duel) && !c.is_attack(duel, &[atk])
}

fn atktg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let atk = duel.get_lp(a.tp) / 2;
    Ok(chk == 0 && duel.is_existing_matching_card(|c, duel| atkfilter(c, duel, atk), a.tp, 0, LOCATION_MZONE, 1, None))
}

fn atkop(duel: &mut Duel, a: &Args) -> Script<()> {
    let atk = duel.get_lp(a.tp);
    let g = duel.get_matching_group(|c, duel| atkfilter(c, duel, atk), a.tp, 0, LOCATION_MZONE, None);
    let c = a.e.get_handler(duel);
    for tc in g.iter() {
        let e1 = duel.create_effect(c);
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_code(duel, EFFECT_SET_ATTACK_FINAL);
        e1.set_value(duel, atk);
        e1.set_reset(duel, RESETS_STANDARD_PHASE_END);
        tc.register_effect(duel, e1);
    }
    Ok(())
}
