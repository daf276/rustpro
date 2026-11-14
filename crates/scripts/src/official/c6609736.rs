// Noh-P.U.N.K. Deer Note
use crate::prelude::*;
use crate::testing::{Check, GRAVE, HAND, MZONE, OPPONENT};

const ID: u32 = 6609736;

pub const LISTED_NAMES: &[u32] = &[ID];
pub const LISTED_SERIES: &[u16] = &[SET_PUNK];

// In the hand with Ze Amin to reveal, on the field for the opponent's Raigeki, and Ze Amin in the Graveyard to summon.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, MZONE),
    (ID, MZONE),
    (19535693, HAND),
    (19535693, HAND),
    (19535693, GRAVE),
    (12580477, HAND | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON + CATEGORY_HANDES);
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_range(duel, LOCATION_HAND);
    e1.set_count_limit(duel, 1, ID);
    e1.set_cost(duel, handspcost);
    e1.set_target(duel, handsptg);
    e1.set_operation(duel, handspop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e2.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e2.set_property(duel, EFFECT_FLAG_DELAY + EFFECT_FLAG_CARD_TARGET);
    e2.set_code(duel, EVENT_TO_GRAVE);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_condition(duel, |duel, a| a.e.get_handler(duel).is_previous_location(duel, LOCATION_ONFIELD));
    e2.set_target(duel, gysptg);
    e2.set_operation(duel, gyspop);
    c.register_effect(duel, e2);
}

fn handspcostfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    if !(c.is_set_card(duel, &[SET_PUNK]) && c.is_monster(duel)) {
        return false;
    }
    let hc = e.get_handler(duel);
    (c.is_can_be_special_summoned(duel, e, 0, tp, false, false) && hc.is_able_to_grave(duel))
        || (hc.is_can_be_special_summoned(duel, e, 0, tp, false, false) && c.is_able_to_grave(duel))
}

fn handspcost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let record: fn(&mut Duel, &Args, CardSet) = |duel, a, og| {
        if let Some(rc) = og.get_first() {
            a.e.set_label_object_group(duel, Some(rc.into()));
            rc.create_effect_relation(duel, a.e);
        }
    };
    Cost::reveal(duel, a, chk, handspcostfilter, true, (1, 1), Some(record), LOCATION_HAND)
}

fn pair(duel: &Duel, e: EffectId) -> CardSet {
    let mut g = CardSet::from(e.get_handler(duel));
    g.merge(e.get_label_object_group(duel).unwrap_or_default());
    g
}

fn handsptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.get_location_count(tp, LOCATION_MZONE) > 0);
    }
    let g = pair(duel, a.e);
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, Some(g), 1, tp, 0);
    duel.set_operation_info(0, CATEGORY_TOGRAVE, Some(g), 1, tp, 0);
    Ok(false)
}

fn handspop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    if duel.get_location_count(tp, LOCATION_MZONE) <= 0 {
        return Ok(());
    }
    let g = pair(duel, e).filter(duel, |c, duel| c.is_relate_to_effect(duel, e), None);
    if g.len() != 2 {
        return Ok(());
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
    let sg = g.filter_select(
        duel,
        tp,
        |c, duel| c.is_can_be_special_summoned(duel, e, 0, tp, false, false),
        1,
        1,
        None,
    )?;
    if !sg.is_empty() && duel.special_summon(sg, 0, tp, tp, false, false, POS_FACEUP)? > 0 {
        duel.send_to_grave(g.difference(sg), REASON_EFFECT)?;
    }
    Ok(())
}

fn gyspfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_set_card(duel, &[SET_PUNK]) && !c.is_level(duel, &[5]) && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
}

fn gysptg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let filter = |c, duel: &Duel| gyspfilter(c, duel, e, tp);
    if let Some(chkc) = chkc {
        return Ok(chkc.is_controler(duel, tp) && chkc.is_location(duel, LOCATION_GRAVE) && filter(chkc, duel));
    }
    if chk == 0 {
        return Ok(duel.get_location_count(tp, LOCATION_MZONE) > 0 && duel.is_existing_target(filter, tp, LOCATION_GRAVE, 0, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
    let g = duel.select_target(tp, filter, tp, LOCATION_GRAVE, 0, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, g, 1, tp, 0);
    Ok(false)
}

fn gyspop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    if let Some(tc) = duel.get_first_target()
        && tc.is_relate_to_effect(duel, a.e)
    {
        duel.special_summon(tc, 0, tp, tp, false, false, POS_FACEUP)?;
    }
    let e1 = duel.create_effect(a.e.get_handler(duel));
    e1.set_description(duel, aux::stringid(ID, 2));
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_property(duel, EFFECT_FLAG_PLAYER_TARGET + EFFECT_FLAG_CLIENT_HINT);
    e1.set_code(duel, EFFECT_CANNOT_SPECIAL_SUMMON);
    e1.set_target_range(duel, 1, 0);
    e1.set_target_card(duel, |duel, _, c| c.is_code(duel, ID));
    e1.set_reset(duel, RESET_PHASE | PHASE_END);
    duel.register_effect(e1, tp);
    Ok(())
}
