// Boot Sector Launch
use crate::prelude::*;
use crate::testing::{Check, GRAVE, HAND, MZONE, OPPONENT};

const ID: u32 = 36668118;

pub const LISTED_SERIES: &[u16] = &[SET_ROKKET];
// In the hand, with Rokket Loaders (Rokket monsters) in the hand and GY to summon, and the opponent's monsters outnumbering ours.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (25554552, HAND),
    (68464358, HAND),
    (25554552, GRAVE),
    (68464358, GRAVE),
    (25554552, GRAVE),
    (13597785, MZONE | OPPONENT),
    (13597785, MZONE | OPPONENT),
    (13597785, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::SpecialSummonedCode(25554552),
    Check::SpecialSummonedCode(68464358),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e0 = duel.create_effect(c);
    e0.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e0.set_code(duel, EVENT_FREE_CHAIN);
    c.register_effect(duel, e0);
    let e1a = duel.create_effect(c);
    e1a.set_type(duel, EFFECT_TYPE_FIELD);
    e1a.set_code(duel, EFFECT_UPDATE_ATTACK);
    e1a.set_range(duel, LOCATION_FZONE);
    e1a.set_target_range(duel, LOCATION_MZONE, LOCATION_MZONE);
    e1a.set_target_card(duel, |duel, _, c| c.is_set_card(duel, &[SET_ROKKET]));
    e1a.set_value(duel, 300);
    c.register_effect(duel, e1a);
    let e1b = e1a.clone_effect(duel);
    e1b.set_code(duel, EFFECT_UPDATE_DEFENSE);
    c.register_effect(duel, e1b);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 0));
    e2.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e2.set_type(duel, EFFECT_TYPE_IGNITION);
    e2.set_range(duel, LOCATION_FZONE);
    e2.set_count_limit(duel, 1, ID);
    e2.set_target(duel, efftg);
    e2.set_operation(duel, effop);
    c.register_effect(duel, e2);
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_set_card(duel, &[SET_ROKKET]) && c.is_can_be_special_summoned_full(duel, e, 0, tp, false, false, POS_FACEUP_DEFENSE, tp, 0xff)
}

fn efftg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let mmz_chk = duel.get_location_count(tp, LOCATION_MZONE) > 0;
    let b1 = mmz_chk && duel.is_existing_matching_card(|c, duel| spfilter(c, duel, e, tp), tp, LOCATION_HAND, 0, 1, None);
    let b2 = mmz_chk
        && duel.get_field_group_count(tp, 0, LOCATION_MZONE) > duel.get_field_group_count(tp, LOCATION_MZONE, 0)
        && duel.is_existing_matching_card(|c, duel| spfilter(c, duel, e, tp), tp, LOCATION_GRAVE, 0, 1, None);
    if chk == 0 {
        return Ok(b1 || b2);
    }
    let op = duel.select_effect(tp, &[(b1, aux::stringid(ID, 1)), (b2, aux::stringid(ID, 2))])?;
    e.set_label(duel, op.map_or(0, |op| op as i64));
    let location = if op == Some(1) { LOCATION_HAND } else { LOCATION_GRAVE };
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, tp, location as i32);
    Ok(false)
}

fn effop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let mut ft = duel.get_location_count(tp, LOCATION_MZONE);
    if ft <= 0 {
        return Ok(());
    }
    let op = e.get_label(duel);
    let location = if op == 1 { LOCATION_HAND } else { LOCATION_GRAVE };
    let g = duel.get_matching_group(|c, duel| spfilter(c, duel, e, tp), tp, location, 0, None);
    if g.is_empty() {
        return Ok(());
    }
    let ct = match op {
        1 => 2,
        2 => duel.get_field_group_count(tp, 0, LOCATION_MZONE) as i32 - duel.get_field_group_count(tp, LOCATION_MZONE, 0) as i32,
        _ => 0,
    };
    ft = ft.min(g.len() as i32).min(ct);
    if ft <= 0 {
        return Ok(());
    }
    if duel.is_player_affected_by_effect(tp, CARD_BLUEEYES_SPIRIT).is_some() {
        ft = 1;
    }
    let sg = aux::select_unselect_group(duel, g, 1, ft as usize, &aux::dncheck, tp, HINTMSG_SPSUMMON)?;
    if !sg.is_empty() {
        duel.special_summon(sg, 0, tp, tp, false, false, POS_FACEUP_DEFENSE)?;
    }
    Ok(())
}
