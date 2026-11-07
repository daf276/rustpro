// Destiny HERO - Dominance
use crate::prelude::*;
use crate::testing::{Check, EXTRA, GRAVE, HAND, MZONE, OPPONENT};

const ID: u32 = 69394324;
pub const MATERIAL_SETCODE: &[u16] = &[SET_HERO, SET_DESTINY_HERO];

struct Materials;

impl FusionMix for Materials {
    const SUB: bool = true;
    const INSF: bool = true;
    const MATERIALS: &'static [FusionMaterial] =
        &[FusionMaterial::Filter(|c, duel, fc, sumtype, tp| c.is_set_card_full(duel, &[SET_DESTINY_HERO], Some(fc), sumtype, tp)); 3];
}

// In the Extra Deck, Fusion Summoned by Polymerization with three Destiny HERO monsters, with Destiny HERO monsters of
// different names in the GY, Kojikocy (a Normal Monster) to battle and destroy, and Destroyer Phoenix Enforcer to destroy it.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, MZONE),
    (ID, MZONE),
    (24094653, HAND),
    (24094653, HAND),
    (9411399, MZONE),
    (9411399, HAND),
    (40591390, MZONE),
    (9411399, GRAVE),
    (40591390, GRAVE),
    (60461804, GRAVE),
    (1184620, MZONE | OPPONENT),
    (60461804, MZONE),
    (1184620, MZONE | OPPONENT),
    (1184620, MZONE | OPPONENT),
    (1184620, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
    Check::ResolvesEffect(aux::stringid(ID, 2)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Fusion::add_proc_mix_n::<Materials>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_count_limit(duel, 1, ID);
    e1.set_target(duel, sttg);
    e1.set_operation(duel, stop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_DRAW);
    e2.set_property(duel, EFFECT_FLAG_PLAYER_TARGET);
    e2.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e2.set_code(duel, EVENT_BATTLE_DESTROYING);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_condition(duel, aux::bdocon);
    e2.set_target(duel, drtg);
    e2.set_operation(duel, drop);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 2));
    e3.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e3.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e3.set_property(
        duel,
        EFFECT_FLAG_CARD_TARGET + EFFECT_FLAG_DELAY + EFFECT_FLAG_DAMAGE_STEP + EFFECT_FLAG_DAMAGE_CAL,
    );
    e3.set_code(duel, EVENT_DESTROYED);
    e3.set_count_limit(duel, 1, (ID, 2));
    e3.set_condition(duel, spcon);
    e3.set_target(duel, sptg);
    e3.set_operation(duel, spop);
    c.register_effect(duel, e3);
}

fn sttg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    Ok(chk == 0 && (duel.get_field_group_count(tp, LOCATION_DECK, 0) > 4 || duel.get_field_group_count(tp, 0, LOCATION_DECK) > 4))
}

fn stop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    let b1 = duel.get_field_group_count(tp, LOCATION_DECK, 0) > 4;
    let b2 = duel.get_field_group_count(tp, 0, LOCATION_DECK) > 4;
    if !(b1 || b2) {
        return Ok(());
    }
    let op = duel
        .select_effect(tp, &[(b1, aux::stringid(ID, 3)), (b2, aux::stringid(ID, 4))])?
        .map_or(0, |op| op as u8);
    duel.sort_decktop(tp, if tp == 0 { op - 1 } else { 2 - op }, 5)
}

fn drtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.is_player_can_draw(tp, 1));
    }
    duel.set_target_player(tp);
    duel.set_target_param(1);
    duel.set_operation_info(0, CATEGORY_DRAW, None, 0, tp, 1);
    Ok(false)
}

fn drop(duel: &mut Duel, _: &Args) -> Script<()> {
    let p: u8 = duel.get_chain_info(0, CHAININFO_TARGET_PLAYER);
    let d: u32 = duel.get_chain_info(0, CHAININFO_TARGET_PARAM);
    duel.draw(p, d, REASON_EFFECT)?;
    Ok(())
}

fn spcon(duel: &Duel, a: &Args) -> bool {
    let c = a.e.get_handler(duel);
    c.is_previous_location(duel, LOCATION_MZONE) && c.is_fusion_summoned(duel) && c.is_reason(duel, REASON_BATTLE | REASON_EFFECT)
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_set_card(duel, &[SET_DESTINY_HERO])
        && c.is_level_below(duel, 9)
        && c.is_can_be_effect_target(duel, e)
        && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if chkc.is_some() {
        return Ok(false);
    }
    let g = duel.get_matching_group(|c, duel| spfilter(c, duel, e, tp), tp, LOCATION_GRAVE, 0, None);
    if chk == 0 {
        return Ok(duel.get_location_count(tp, LOCATION_MZONE) > 2
            && duel.is_player_affected_by_effect(tp, CARD_BLUEEYES_SPIRIT).is_none()
            && g.get_class_count(duel, |c, duel| i64::from(c.get_code(duel))) > 2);
    }
    let sg = aux::select_unselect_group(duel, g, 3, 3, &aux::dncheck, tp, HINTMSG_SPSUMMON)?;
    duel.set_target_card(sg);
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, Some(sg), 3, 0, 0);
    Ok(false)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let ft = duel.get_location_count(tp, LOCATION_MZONE);
    if ft <= 0 || duel.is_player_affected_by_effect(tp, CARD_BLUEEYES_SPIRIT).is_some() {
        return Ok(());
    }
    let Some(g) = duel.get_target_cards(e).filter(|g| !g.is_empty()) else {
        return Ok(());
    };
    if g.len() <= ft as usize {
        duel.special_summon(g, 0, tp, tp, false, false, POS_FACEUP)?;
    } else {
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
        let sg = g.select(duel, tp, ft as u8, ft as u8, None)?;
        duel.special_summon(sg, 0, tp, tp, false, false, POS_FACEUP)?;
    }
    Ok(())
}
