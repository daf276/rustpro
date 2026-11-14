// Nebulus, Dragon Ruler of Mishaps
use super::c89185742::cannot_attack_this_turn;
use crate::prelude::*;
use crate::testing::{Check, GRAVE, HAND, REMOVED};

const ID: u32 = 94655777;
const ECLEPSIS: u32 = 30350202;

pub const LISTED_SERIES: &[u16] = &[SET_DRAGON_RULER];
pub const LISTED_NAMES: &[u32] = &[ID, ECLEPSIS];

// Banished Dragon Rulers of three Attributes (banished face-down half the time), Red-Eyes Black Dragon (a DARK Dragon) to
// discard and to banish from the Graveyard, and a copy with Eclepsis in the Graveyard.
pub const SUPPORT: &[(u32, u16)] = &[
    (89399912, REMOVED),
    (89399912, REMOVED),
    (53804307, REMOVED),
    (53804307, REMOVED),
    (26400609, REMOVED),
    (26400609, REMOVED),
    (74677422, HAND),
    (74677422, GRAVE),
    (ID, GRAVE),
    (ECLEPSIS, GRAVE),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e1.set_range(duel, LOCATION_HAND);
    e1.set_count_limit(duel, 1, ID);
    e1.set_cost(duel, DragonRuler::self_discard_cost::<ATTRIBUTE_DARK>);
    e1.set_target(duel, rmsptg);
    e1.set_operation(duel, rmspop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e2.set_type(duel, EFFECT_TYPE_IGNITION);
    e2.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e2.set_range(duel, LOCATION_GRAVE);
    e2.set_count_limit(duel, 1, ID);
    e2.set_cost(duel, gyspcost);
    e2.set_target(duel, gysptg);
    e2.set_operation(duel, gyspop);
    c.register_effect(duel, e2);
}

fn rmspfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_set_card(duel, &[SET_DRAGON_RULER])
        && c.is_faceup(duel)
        && c.is_can_be_effect_target(duel, e)
        && !c.is_code(duel, ID)
        && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
}

fn rmsptg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if chkc.is_some() {
        return Ok(false);
    }
    let g = duel.get_matching_group(|c, duel| rmspfilter(c, duel, e, tp), tp, LOCATION_REMOVED, 0, None);
    if chk == 0 {
        return Ok(duel.is_player_affected_by_effect(tp, CARD_BLUEEYES_SPIRIT).is_none()
            && duel.get_location_count(tp, LOCATION_MZONE) >= 2
            && g.get_class_count(duel, |c, duel| i64::from(c.get_attribute(duel))) >= 2);
    }
    let tg = aux::select_unselect_group(
        duel,
        g,
        2,
        2,
        &aux::dpcheck(|c, duel| i64::from(c.get_attribute(duel))),
        tp,
        HINTMSG_SPSUMMON,
    )?;
    duel.set_target_card(tg);
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, Some(tg), 2, tp, 0);
    Ok(false)
}

fn rmspop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let ft = duel.get_location_count(tp, LOCATION_MZONE);
    if ft <= 0 {
        return Ok(());
    }
    let mut tg = duel.get_target_cards(e).unwrap_or_default();
    if tg.is_empty() || duel.is_player_affected_by_effect(tp, CARD_BLUEEYES_SPIRIT).is_some() && tg.len() > 1 {
        return Ok(());
    }
    if tg.len() > ft as usize {
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
        tg = tg.select(duel, tp, ft as u8, ft as u8, None)?;
    }
    let c = e.get_handler(duel);
    for tc in tg {
        if duel.special_summon_step(tc, 0, tp, tp, false, false, POS_FACEUP)? {
            cannot_attack_this_turn(duel, c, tc);
        }
    }
    duel.special_summon_complete()?;
    Ok(())
}

fn gyspcostfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    (c.is_attribute(duel, ATTRIBUTE_DARK) || c.is_race(duel, RACE_DRAGON))
        && c.is_able_to_remove_as_cost(duel)
        && duel.is_existing_target(|c, duel| gyspfilter(c, duel, e, tp), tp, LOCATION_GRAVE, 0, 1, Some(c.into()))
}

fn gyspfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_code(duel, ECLEPSIS) && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
}

fn gyspcost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    let filter = |sc: CardId, duel: &Duel| gyspcostfilter(sc, duel, e, tp);
    if chk == 0 {
        return Ok(c.is_able_to_remove_as_cost(duel) && duel.is_existing_matching_card(filter, tp, LOCATION_GRAVE, 0, 1, Some(c.into())));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_REMOVE);
    let mut g = duel.select_matching_card(tp, filter, tp, LOCATION_GRAVE, 0, 1, 1, Some(c.into()))?;
    g.insert(c);
    duel.remove(g, POS_FACEUP, REASON_COST)?;
    Ok(false)
}

fn gysptg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_GRAVE) && chkc.is_controler(duel, tp) && gyspfilter(chkc, duel, e, tp));
    }
    if chk == 0 {
        // The Lua passes an undefined global `c` as the exception, so nil.
        return Ok(duel.get_location_count(tp, LOCATION_MZONE) > 0
            && duel.is_existing_target(|c, duel| gyspfilter(c, duel, e, tp), tp, LOCATION_GRAVE, 0, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
    let g = duel.select_target(tp, |c, duel| gyspfilter(c, duel, e, tp), tp, LOCATION_GRAVE, 0, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, g, 1, tp, 0);
    Ok(false)
}

fn gyspop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    if let Some(tc) = duel.get_first_target().filter(|tc| tc.is_relate_to_effect(duel, a.e)) {
        duel.special_summon(tc, 0, tp, tp, false, false, POS_FACEUP)?;
    }
    Ok(())
}
