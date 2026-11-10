// Yubel - The Loving Defender Forever
use crate::prelude::*;
use crate::testing::{Check, EXTRA, HAND, MZONE, OPPONENT};

const ID: u32 = 47172959;
const BATTLED_MONSTER: u8 = 0;
const ATK_ON_FIELD: u8 = 1;

struct Materials;

impl FusionMixRep for Materials {
    const SUB: bool = true;
    const INSF: bool = true;
    const FUN1: FusionMaterial =
        FusionMaterial::Filter(|c, duel, fc, sumtype, tp| c.is_type_full(duel, TYPE_EFFECT, Some(fc), sumtype, tp) && c.is_on_field(duel));
    const MINC: i32 = 1;
    const MAXC: i32 = 99;
    const MATERIALS: &'static [FusionMaterial] = &[FusionMaterial::Filter(|c, duel, fc, sumtype, tp| {
        c.is_set_card_full(duel, &[SET_YUBEL], Some(fc), sumtype, tp)
    })];
}

// Fusion Summoned by Polymerization from Yubel and Effect Monsters on the field, with Sangans of the opponent's to fight.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (CARD_POLYMERIZATION, HAND),
    (CARD_POLYMERIZATION, HAND),
    (78371393, MZONE),
    (26202165, MZONE),
    (14558127, MZONE),
    (26202165, MZONE | OPPONENT),
    (26202165, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Fusion::add_proc_mix_rep::<Materials>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_DAMAGE);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e1.set_property(duel, EFFECT_FLAG_DELAY + EFFECT_FLAG_PLAYER_TARGET);
    e1.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e1.set_condition(duel, |duel, a| a.e.get_handler(duel).is_fusion_summoned(duel));
    e1.set_target(duel, damtg);
    e1.set_operation(duel, damop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_SINGLE);
    e2.set_property(duel, EFFECT_FLAG_SINGLE_RANGE);
    e2.set_code(duel, EFFECT_INDESTRUCTABLE_BATTLE);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_value(duel, 1);
    c.register_effect(duel, e2);
    let e3 = e2.clone_effect(duel);
    e3.set_code(duel, EFFECT_INDESTRUCTABLE_EFFECT);
    c.register_effect(duel, e3);
    let e4 = e2.clone_effect(duel);
    e4.set_code(duel, EFFECT_AVOID_BATTLE_DAMAGE);
    c.register_effect(duel, e4);
    let e5 = duel.create_effect(c);
    e5.set_description(duel, aux::stringid(ID, 1));
    e5.set_category(duel, CATEGORY_DAMAGE + CATEGORY_REMOVE);
    e5.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_F);
    e5.set_code(duel, EVENT_DAMAGE_STEP_END);
    e5.set_condition(duel, damrmcon);
    e5.set_target(duel, damrmtg);
    e5.set_operation(duel, damrmop);
    c.register_effect(duel, e5);
}

fn damtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    let mg = a.e.get_handler(duel).get_material(duel);
    if chk == 0 {
        return Ok(!mg.is_empty());
    }
    let dmg = mg.len() as u32 * 500;
    duel.set_target_player(1 - tp);
    duel.set_target_param(dmg);
    duel.set_operation_info(0, CATEGORY_DAMAGE, None, 0, 1 - tp, dmg as i32);
    Ok(false)
}

fn damop(duel: &mut Duel, _: &Args) -> Script<()> {
    let p: u8 = duel.get_chain_info(0, CHAININFO_TARGET_PLAYER);
    let d: u32 = duel.get_chain_info(0, CHAININFO_TARGET_PARAM);
    duel.damage(p, d, REASON_EFFECT)?;
    Ok(())
}

fn damrmcon(duel: &Duel, a: &Args) -> bool {
    let Some(bc) = a.e.get_handler(duel).get_battle_target(duel) else {
        return false;
    };
    if bc.is_relate_to_battle(duel) {
        bc.is_controler(duel, 1 - a.tp)
    } else {
        bc.is_previous_controler(duel, 1 - a.tp)
    }
}

fn damrmtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(true);
    }
    let (e, tp) = (a.e, a.tp);
    let Some(bc) = e.get_handler(duel).get_battle_target(duel) else {
        return Ok(false);
    };
    let dam = if bc.is_relate_to_battle(duel) {
        duel.set_operation_info(0, CATEGORY_REMOVE, Some(bc.into()), 1, 0, 0);
        bc.get_attack(duel)
    } else {
        let dam = bc.get_previous_attack_on_field(duel);
        e.set_chain_data(duel, ATK_ON_FIELD, dam.into());
        dam
    };
    e.set_chain_data_group(duel, BATTLED_MONSTER, bc.into());
    duel.set_operation_info(0, CATEGORY_DAMAGE, None, 0, 1 - tp, dam);
    Ok(false)
}

fn damrmop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let Some(bc) = e.get_chain_data_group(duel, BATTLED_MONSTER).and_then(|g| g.get_first()) else {
        return Ok(());
    };
    let battle_relation = bc.is_relate_to_battle(duel);
    let dam = if battle_relation && bc.is_faceup(duel) && bc.is_controler(duel, 1 - tp) {
        bc.get_attack(duel)
    } else if !battle_relation {
        e.get_chain_data(duel, ATK_ON_FIELD).unwrap_or(0) as i32
    } else {
        0
    };
    if duel.damage(1 - tp, dam.max(0) as u32, REASON_EFFECT)? > 0 && battle_relation {
        duel.remove(bc, POS_FACEUP, REASON_EFFECT)?;
    }
    Ok(())
}
