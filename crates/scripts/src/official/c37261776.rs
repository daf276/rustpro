// Salamangreat Violet Chimera
use crate::prelude::*;
use crate::testing::{Check, EXTRA, HAND, MZONE};

const ID: u32 = 37261776;

pub const LISTED_SERIES: &[u16] = &[SET_SALAMANGREAT];
pub const MATERIAL_SETCODE: &[u16] = &[SET_SALAMANGREAT];
pub const LISTED_NAMES: &[u32] = &[ID];

struct Materials;

impl FusionMix for Materials {
    const SUB: bool = true;
    const INSF: bool = true;
    const MATERIALS: &'static [FusionMaterial] = &[
        FusionMaterial::Filter(|c, duel, fc, sumtype, tp| c.is_set_card_full(duel, &[SET_SALAMANGREAT], Some(fc), sumtype, tp)),
        FusionMaterial::Filter(|c, duel, fc, sumtype, tp| c.is_type_full(duel, TYPE_LINK, Some(fc), sumtype, tp)),
    ];
}

// In the Extra Deck and on the field, with Polymerization and Salamangreat Gazelle in the hand and Salamangreat Pyro Phoenix, a Link
// Monster, on the field as materials, and Yami changing the ATK of the opponent's Spellcasters to double against.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (ID, MZONE),
    (24094653, HAND),
    (24094653, HAND),
    (26889158, HAND),
    (31313405, MZONE),
    (31313405, MZONE),
    (59197169, 0),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    aux::enable_check_reincarnation(duel);
    Fusion::add_proc_mix::<Materials>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_ATKCHANGE);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e1.set_property(duel, EFFECT_FLAG_DELAY);
    e1.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e1.set_condition(duel, |duel, a| {
        a.e.get_handler(duel).is_fusion_summoned(duel) && a.e.get_label(duel) > 0
    });
    e1.set_target(duel, sumtg);
    e1.set_operation(duel, sumop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_SINGLE);
    e2.set_code(duel, EFFECT_MATERIAL_CHECK);
    e2.set_value(duel, Value::MaterialCheck(valcheck));
    e2.set_label_object(duel, Some(e1));
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 1));
    e3.set_category(duel, CATEGORY_ATKCHANGE);
    e3.set_type(duel, EFFECT_TYPE_QUICK_O);
    e3.set_code(duel, EVENT_PRE_DAMAGE_CALCULATE);
    e3.set_range(duel, LOCATION_MZONE);
    e3.set_condition(duel, atkcon);
    e3.set_target(duel, atktg);
    e3.set_operation(duel, atkop);
    c.register_effect(duel, e3);
    let e4 = duel.create_effect(c);
    e4.set_type(duel, EFFECT_TYPE_FIELD);
    e4.set_code(duel, EFFECT_SET_ATTACK);
    e4.set_range(duel, LOCATION_MZONE);
    e4.set_target_range(duel, LOCATION_MZONE, LOCATION_MZONE);
    e4.set_condition(duel, zeroatkcon);
    e4.set_target_card(duel, |duel, e, c| e.get_handler(duel).get_battle_target(duel) == Some(c));
    e4.set_value(duel, 0);
    c.register_effect(duel, e4);
}

// The Lua's label/2, rounded as the core reads it.
fn half_label(duel: &Duel, e: EffectId) -> i32 {
    ((e.get_label(duel) + 1) / 2) as i32
}

fn sumtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(true);
    }
    let half = half_label(duel, a.e);
    duel.set_operation_info(0, CATEGORY_ATKCHANGE, Some(a.e.get_handler(duel).into()), 1, a.tp, half);
    Ok(false)
}

fn sumop(duel: &mut Duel, a: &Args) -> Script<()> {
    let c = a.e.get_handler(duel);
    if c.is_faceup(duel) && c.is_relate_to_effect(duel, a.e) {
        let half = half_label(duel, a.e);
        let e1 = duel.create_effect(c);
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_property(duel, EFFECT_FLAG_SINGLE_RANGE);
        e1.set_code(duel, EFFECT_UPDATE_ATTACK);
        e1.set_range(duel, LOCATION_MZONE);
        e1.set_value(duel, half);
        e1.set_reset(duel, RESETS_STANDARD_DISABLE_PHASE_END);
        c.register_effect(duel, e1);
    }
    Ok(())
}

fn valcheck(duel: &mut Duel, e: EffectId, c: CardId) {
    let g = c.get_material(duel);
    let atk = if g.is_empty() {
        0
    } else {
        g.get_sum(duel, |mc, duel| i64::from(mc.get_base_attack(duel)))
    };
    if let Some(le) = e.get_label_object(duel) {
        le.set_label(duel, atk);
    }
}

fn atkcon(duel: &Duel, a: &Args) -> bool {
    a.e.get_handler(duel)
        .get_battle_target(duel)
        .is_some_and(|bc| bc.get_attack(duel) != bc.get_base_attack(duel))
}

fn atktg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let c = a.e.get_handler(duel);
    if chk == 0 {
        return Ok(c.get_flag_effect(duel, ID) == 0);
    }
    c.register_flag_effect(
        duel,
        ID,
        RESET_EVENT | RESETS_STANDARD | RESET_PHASE | PHASE_DAMAGE_CAL | RESET_CHAIN,
        0,
        1,
    );
    let atk = c.get_attack(duel);
    duel.set_operation_info(0, CATEGORY_ATKCHANGE, Some(c.into()), 1, a.tp, atk);
    Ok(false)
}

fn atkop(duel: &mut Duel, a: &Args) -> Script<()> {
    let c = a.e.get_handler(duel);
    if c.is_faceup(duel) && c.is_relate_to_effect(duel, a.e) {
        let atk = c.get_attack(duel) * 2;
        let e1 = duel.create_effect(c);
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_property(duel, EFFECT_FLAG_SINGLE_RANGE);
        e1.set_code(duel, EFFECT_SET_ATTACK_FINAL);
        e1.set_range(duel, LOCATION_MZONE);
        e1.set_value(duel, atk);
        e1.set_reset(duel, RESET_EVENT | RESETS_STANDARD_DISABLE | RESET_PHASE | PHASE_DAMAGE_CAL);
        c.register_effect(duel, e1);
    }
    Ok(())
}

fn zeroatkcon(duel: &Duel, a: &Args) -> bool {
    let c = a.e.get_handler(duel);
    c.is_reincarnation_summoned(duel) && c.is_fusion_summoned(duel) && duel.is_phase(PHASE_DAMAGE_CAL)
}
