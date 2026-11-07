// Elemental HERO Shining Neos Wingman
use crate::prelude::*;
use crate::testing::{Check, EXTRA, HAND, MZONE, OPPONENT};

const ID: u32 = 56733747;
pub const MATERIAL_SETCODE: &[u16] = &[SET_HERO, SET_ELEMENTAL_HERO, SET_NEOS, SET_WINGMAN];

pub const LISTED_NAMES: &[u32] = &[CARD_NEOS];

struct Materials;

impl FusionMix for Materials {
    const SUB: bool = true;
    const INSF: bool = true;
    const MATERIALS: &'static [FusionMaterial] = &[FusionMaterial::Code(CARD_NEOS), FusionMaterial::Filter(ffilter)];
}

// In the Extra Deck, Fusion Summoned by Polymerization with Elemental HERO Neos (a Normal Monster) and Flame Wingman, and on
// the field to battle weak Sangan, with a monster in the GY for its ATK.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, MZONE),
    (CARD_POLYMERIZATION, HAND),
    (CARD_NEOS, HAND),
    (35809262, MZONE),
    (26202165, MZONE | OPPONENT),
    (26202165, MZONE | OPPONENT),
    (ID, MZONE),
    (26202165, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::SpecialSummoned,
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Fusion::add_proc_mix::<Materials>(duel, c);
    c.add_must_be_fusion_summoned(duel);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_DESTROY);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e1.set_property(duel, EFFECT_FLAG_DELAY);
    e1.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e1.set_count_limit(duel, 1, ID);
    e1.set_target(duel, destg);
    e1.set_operation(duel, desop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_SINGLE);
    e2.set_code(duel, EFFECT_UPDATE_ATTACK);
    e2.set_property(duel, EFFECT_FLAG_SINGLE_RANGE);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_value(duel, Value::Card(atkval));
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_type(duel, EFFECT_TYPE_SINGLE);
    e3.set_code(duel, EFFECT_INDESTRUCTABLE_EFFECT);
    e3.set_property(duel, EFFECT_FLAG_SINGLE_RANGE);
    e3.set_range(duel, LOCATION_MZONE);
    e3.set_value(duel, 1);
    c.register_effect(duel, e3);
    let e4 = duel.create_effect(c);
    e4.set_description(duel, aux::stringid(ID, 1));
    e4.set_category(duel, CATEGORY_DAMAGE);
    e4.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_F);
    e4.set_property(duel, EFFECT_FLAG_PLAYER_TARGET);
    e4.set_code(duel, EVENT_BATTLE_DESTROYING);
    e4.set_condition(duel, aux::bdcon);
    e4.set_target(duel, damtg);
    e4.set_operation(duel, damop);
    c.register_effect(duel, e4);
}

fn ffilter(c: CardId, duel: &Duel, fc: CardId, sumtype: u64, tp: u8) -> bool {
    c.is_type_full(duel, TYPE_FUSION, Some(fc), sumtype, tp) && c.is_set_card_full(duel, &[SET_WINGMAN], Some(fc), sumtype, tp)
}

fn attribute_count(duel: &Duel, tp: u8) -> usize {
    let attg = duel.get_matching_group(CardId::is_faceup, tp, LOCATION_MZONE, LOCATION_MZONE, None);
    attg.get_class_count(duel, |c, duel| i64::from(c.get_attribute(duel)))
}

fn destg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    let ct = attribute_count(duel, tp);
    let dg = duel.get_matching_group(|_, _| true, tp, 0, LOCATION_ONFIELD, None);
    if chk == 0 {
        return Ok(ct > 0 && !dg.is_empty());
    }
    duel.set_operation_info(0, CATEGORY_DESTROY, Some(dg), 1, 0, 0);
    Ok(false)
}

fn desop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    let attg = duel.get_matching_group(CardId::is_faceup, tp, LOCATION_MZONE, LOCATION_MZONE, None);
    if attg.is_empty() {
        return Ok(());
    }
    let ct = attribute_count(duel, tp);
    if ct > 0 {
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_DESTROY);
        let dg = duel.select_matching_card(tp, |_, _| true, tp, 0, LOCATION_ONFIELD, 1, ct as u8, None)?;
        if dg.is_empty() {
            return Ok(());
        }
        duel.hint_selection(dg);
        duel.destroy(dg, REASON_EFFECT)?;
    }
    Ok(())
}

fn atkval(duel: &Duel, e: EffectId, _: CardId) -> i32 {
    duel.get_matching_group_count(CardId::is_monster, e.get_handler_player(duel), LOCATION_GRAVE, 0, None) as i32 * 300
}

fn damtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(true);
    }
    let dam =
        a.e.get_handler(duel)
            .get_battle_target(duel)
            .map_or(0, |bc| bc.get_base_attack(duel).max(0));
    duel.set_target_player(1 - tp);
    duel.set_target_param(dam as u32);
    duel.set_operation_info(0, CATEGORY_DAMAGE, None, 0, 1 - tp, dam);
    Ok(false)
}

fn damop(duel: &mut Duel, _: &Args) -> Script<()> {
    let p: u8 = duel.get_chain_info(0, CHAININFO_TARGET_PLAYER);
    let d: u32 = duel.get_chain_info(0, CHAININFO_TARGET_PARAM);
    duel.damage(p, d, REASON_EFFECT)?;
    Ok(())
}
