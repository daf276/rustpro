// Jiaotu, Darkness of the Yang Zing
use super::c38814750::{rmcon, rmop};
use super::c61488417::condition;
use crate::prelude::*;
use crate::testing::{Check, DECK, HAND};

const ID: u32 = 25935625;
const CHIWEN: u32 = 61488417;

pub const LISTED_NAMES: &[u32] = &[ID];

// Two Chiwens in the hand to discard and four in the Deck to Special Summon, and Dark Hole in the Deck to destroy it.
pub const SUPPORT: &[(u32, u16)] = &[
    (CHIWEN, HAND),
    (CHIWEN, HAND),
    (CHIWEN, DECK),
    (CHIWEN, DECK),
    (CHIWEN, DECK),
    (CHIWEN, DECK),
    (53129443, DECK),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e1.set_property(duel, EFFECT_FLAG_DAMAGE_STEP);
    e1.set_code(duel, EVENT_TO_GRAVE);
    e1.set_count_limit(duel, 1, ID);
    e1.set_condition(duel, condition);
    e1.set_target(duel, target);
    e1.set_operation(duel, operation);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e2.set_type(duel, EFFECT_TYPE_IGNITION);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_count_limit(duel, 1, ID);
    e2.set_condition(duel, |duel, a| duel.get_field_group_count(a.tp, LOCATION_MZONE, 0) == 1);
    e2.set_cost(duel, spcost);
    e2.set_target(duel, sptg);
    e2.set_operation(duel, spop);
    c.register_effect(duel, e2);
}

fn filter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_set_card(duel, &[SET_YANG_ZING]) && !c.is_code(duel, ID) && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
}

fn target(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if chk == 0 {
        return Ok(duel.get_location_count(tp, LOCATION_MZONE) > 0
            && duel.is_existing_matching_card(|c, duel| filter(c, duel, e, tp), tp, LOCATION_DECK, 0, 1, None));
    }
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, tp, LOCATION_DECK as i32);
    Ok(false)
}

fn operation(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    if duel.get_location_count(tp, LOCATION_MZONE) <= 0 {
        return Ok(());
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
    let g = duel.select_matching_card(tp, |c, duel| filter(c, duel, e, tp), tp, LOCATION_DECK, 0, 1, 1, None)?;
    if !g.is_empty() {
        duel.special_summon(g, 0, tp, tp, false, false, POS_FACEUP_ATTACK)?;
    }
    Ok(())
}

fn cfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_YANG_ZING]) && c.is_able_to_grave_as_cost(duel)
}

fn spcost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(cfilter, tp, LOCATION_HAND, 0, 2, None));
    }
    duel.discard_hand(tp, cfilter, 2, 2, REASON_COST)?;
    Ok(false)
}

fn spfilter1(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_set_card(duel, &[SET_YANG_ZING])
        && c.get_attack(duel) == 0
        && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
        && duel.is_existing_matching_card(|sc, duel| spfilter2(sc, duel, e, tp), tp, LOCATION_DECK, 0, 1, Some(c.into()))
}

fn spfilter2(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_set_card(duel, &[SET_YANG_ZING]) && c.is_defense_below(duel, 0) && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if chk == 0 {
        return Ok(duel.is_player_affected_by_effect(tp, CARD_BLUEEYES_SPIRIT).is_none()
            && duel.get_location_count(tp, LOCATION_MZONE) > 1
            && duel.is_existing_matching_card(|c, duel| spfilter1(c, duel, e, tp), tp, LOCATION_DECK, 0, 1, None));
    }
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 2, tp, LOCATION_DECK as i32);
    Ok(false)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    if duel.is_player_affected_by_effect(tp, CARD_BLUEEYES_SPIRIT).is_some() || duel.get_location_count(tp, LOCATION_MZONE) < 2 {
        return Ok(());
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
    let mut g1 = duel.select_matching_card(tp, |c, duel| spfilter1(c, duel, e, tp), tp, LOCATION_DECK, 0, 1, 1, None)?;
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
    let g2 = duel.select_matching_card(
        tp,
        |c, duel| spfilter2(c, duel, e, tp),
        tp,
        LOCATION_DECK,
        0,
        1,
        1,
        g1.get_first().map(CardSet::from),
    )?;
    g1.merge(g2);
    if g1.is_empty() {
        return Ok(());
    }
    let c = e.get_handler(duel);
    let fid = c.get_field_id(duel);
    for tc in g1 {
        duel.special_summon_step(tc, 0, tp, tp, false, false, POS_FACEUP)?;
        tc.register_flag_effect_full(duel, ID, RESET_EVENT | RESETS_STANDARD, 0, 1, fid.into(), 0);
    }
    duel.special_summon_complete()?;
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_CONTINUOUS);
    e1.set_code(duel, EVENT_PHASE + PHASE_END);
    e1.set_property(duel, EFFECT_FLAG_IGNORE_IMMUNE);
    e1.set_count_limit(duel, 1, 0);
    e1.set_label(duel, fid.into());
    e1.set_label_object_group(duel, Some(g1));
    e1.set_condition_mut(duel, rmcon::<ID>);
    e1.set_operation(duel, rmop::<ID>);
    duel.register_effect(e1, tp);
    Ok(())
}
