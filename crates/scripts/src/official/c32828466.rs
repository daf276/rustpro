// Wake Up Your Elemental HERO
use crate::prelude::*;
use crate::testing::{Check, DECK, EXTRA, HAND, MZONE, OPPONENT};

const ID: u32 = 32828466;

struct Materials;

impl FusionMixRep for Materials {
    const SUB: bool = true;
    const INSF: bool = true;
    const FUN1: FusionMaterial =
        FusionMaterial::Filter(|c, duel, fc, sumtype, tp| c.is_race_full(duel, RACE_WARRIOR, Some(fc), sumtype, tp));
    const MINC: i32 = 1;
    const MAXC: i32 = 99;
    const MATERIALS: &'static [FusionMaterial] = &[FusionMaterial::Filter(mfilter)];
}

pub const LISTED_SERIES: &[u16] = &[SET_ELEMENTAL_HERO];
pub const MATERIAL_SETCODE: &[u16] = &[SET_HERO, SET_ELEMENTAL_HERO];

// In the Extra Deck, Fusion Summoned by Polymerization with Elemental HERO Flame Wingman and Doron, Warriors in the Deck to
// summon, and the opponent's monsters to battle and Dark Hole to destroy it.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (CARD_POLYMERIZATION, HAND),
    (35809262, MZONE),
    (35809262, MZONE),
    (756652, HAND),
    (756652, MZONE),
    (756652, DECK),
    (13597785, MZONE | OPPONENT),
    (53129443, HAND | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::SpecialSummoned,
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Fusion::add_proc_mix_rep::<Materials>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE + EFFECT_FLAG_UNCOPYABLE);
    e1.set_code(duel, EFFECT_SPSUMMON_CONDITION);
    e1.set_value(duel, Value::SpSummonCondition(aux::fuslimit));
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_SINGLE);
    e2.set_code(duel, EFFECT_MATERIAL_CHECK);
    e2.set_value(duel, Value::MaterialCheck(matcheck));
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 0));
    e3.set_category(duel, CATEGORY_DESTROY + CATEGORY_DAMAGE);
    e3.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_F);
    e3.set_code(duel, EVENT_BATTLED);
    e3.set_condition(duel, descon);
    e3.set_target(duel, destg);
    e3.set_operation(duel, desop);
    c.register_effect(duel, e3);
    let e4 = duel.create_effect(c);
    e4.set_description(duel, aux::stringid(ID, 1));
    e4.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e4.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_F);
    e4.set_code(duel, EVENT_DESTROYED);
    e4.set_condition(duel, spcon);
    e4.set_target(
        duel,
        stock::operation_info_target::<CATEGORY_SPECIAL_SUMMON, { LOCATION_HAND | LOCATION_DECK }>,
    );
    e4.set_operation(duel, spop);
    c.register_effect(duel, e4);
}

fn mfilter(c: CardId, duel: &Duel, fc: CardId, sumtype: u64, tp: u8) -> bool {
    c.is_set_card_full(duel, &[SET_ELEMENTAL_HERO], Some(fc), sumtype, tp) && c.is_type_full(duel, TYPE_FUSION, Some(fc), sumtype, tp)
}

fn matcheck(duel: &mut Duel, _: EffectId, c: CardId) {
    let g = c.get_material(duel);
    let reset = RESET_EVENT | (RESETS_STANDARD_DISABLE & !RESET_TOFIELD);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_code(duel, EFFECT_UPDATE_ATTACK);
    e1.set_value(duel, g.len() as i32 * 300);
    e1.set_reset(duel, reset);
    c.register_effect(duel, e1);
    let fusions = g.filter_count(duel, |c, duel| c.is_type(duel, TYPE_FUSION), None) as i32;
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_SINGLE);
    e2.set_code(duel, EFFECT_EXTRA_ATTACK_MONSTER);
    e2.set_value(duel, fusions - 1);
    e2.set_reset(duel, reset);
    c.register_effect(duel, e2);
}

fn descon(duel: &Duel, a: &Args) -> bool {
    a.e.get_handler(duel)
        .get_battle_target(duel)
        .is_some_and(|bc| bc.is_relate_to_battle(duel))
}

fn destg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(true);
    }
    let bc = a.e.get_handler(duel).get_battle_target(duel);
    duel.set_operation_info(0, CATEGORY_DESTROY, bc.map(CardSet::from), 1, 0, 0);
    let atk = bc.map_or(0, |bc| bc.get_base_attack(duel));
    duel.set_operation_info(0, CATEGORY_DAMAGE, None, 0, 1 - a.tp, atk);
    Ok(false)
}

fn desop(duel: &mut Duel, a: &Args) -> Script<()> {
    let bc = a.e.get_handler(duel).get_battle_target(duel);
    if let Some(bc) = bc.filter(|bc| bc.is_relate_to_battle(duel))
        && duel.destroy(bc, REASON_EFFECT)? > 0
    {
        let atk = bc.get_base_attack(duel);
        duel.damage(1 - a.tp, atk as u32, REASON_EFFECT)?;
    }
    Ok(())
}

fn spcon(duel: &Duel, a: &Args) -> bool {
    let c = a.e.get_handler(duel);
    c.is_previous_location(duel, LOCATION_MZONE) && c.is_fusion_summoned(duel)
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, sp: u8) -> bool {
    c.is_race(duel, RACE_WARRIOR) && c.is_can_be_special_summoned(duel, e, 0, sp, false, false)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    stock::special_summon(
        duel,
        a,
        |c, duel| spfilter(c, duel, a.e, a.tp),
        LOCATION_DECK | LOCATION_HAND,
        POS_FACEUP,
    )
}
