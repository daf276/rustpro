// Goldenhair, the Newest Plunder Patroll
use crate::prelude::*;
use crate::testing::{Check, GRAVE, HAND, MZONE};

const ID: u32 = 81344070;

pub const LISTED_SERIES: &[u16] = &[SET_PLUNDER_PATROLL];
// Whitebeard, the Plunder Patroll Helm in the hand to send to the Graveyard as the cost, and copies in the Graveyard and hand.
pub const SUPPORT: &[(u32, u16)] = &[(ID, HAND), (ID, GRAVE), (31374201, HAND), (31374201, MZONE)];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_range(duel, LOCATION_HAND);
    e1.set_count_limit(duel, 1, ID);
    e1.set_cost(duel, spcost1);
    e1.set_target(duel, sptg1);
    e1.set_operation(duel, spop1);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e2.set_type(duel, EFFECT_TYPE_IGNITION);
    e2.set_range(duel, LOCATION_GRAVE);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_cost(duel, spcost2);
    e2.set_target(duel, sptg2);
    e2.set_operation(duel, spop2);
    c.register_effect(duel, e2);
}

fn spcfilter(c: CardId, duel: &Duel, tp: u8) -> bool {
    c.is_set_card(duel, &[SET_PLUNDER_PATROLL])
        && c.is_original_type(duel, TYPE_MONSTER)
        && c.is_able_to_grave_as_cost(duel)
        && (c.is_faceup(duel) || c.is_location(duel, LOCATION_HAND))
        && duel.get_mzone_count(tp, Some(c.into())) > 0
}

fn spcost1(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let exception = Some(e.get_handler(duel).into());
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(
            |c, duel| spcfilter(c, duel, tp),
            tp,
            LOCATION_HAND | LOCATION_ONFIELD,
            0,
            1,
            exception,
        ));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TOGRAVE);
    let tc = duel.select_matching_card(
        tp,
        |c, duel| spcfilter(c, duel, tp),
        tp,
        LOCATION_HAND | LOCATION_ONFIELD,
        0,
        1,
        1,
        exception,
    )?;
    duel.send_to_grave(tc, REASON_COST)?;
    Ok(false)
}

fn sptg1(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if chk == 0 {
        return Ok(c.is_can_be_special_summoned(duel, e, 0, tp, false, false));
    }
    let location = c.get_location(duel);
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, Some(c.into()), 1, 0, location as i32);
    Ok(false)
}

fn spop1(duel: &mut Duel, a: &Args) -> Script<()> {
    let c = a.e.get_handler(duel);
    if !c.is_relate_to_effect(duel, a.e) {
        return Ok(());
    }
    duel.special_summon(c, 0, a.tp, a.tp, false, false, POS_FACEUP)?;
    Ok(())
}

fn spcost2(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(CardId::is_discardable, tp, LOCATION_HAND, 0, 1, None));
    }
    duel.discard_hand(tp, CardId::is_discardable, 1, 1, REASON_COST | REASON_DISCARD)?;
    Ok(false)
}

fn sptg2(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if chk == 0 {
        return Ok(duel.get_location_count(tp, LOCATION_MZONE) > 0 && c.is_can_be_special_summoned(duel, e, 0, tp, false, false));
    }
    let location = c.get_location(duel);
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, Some(c.into()), 1, 0, location as i32);
    Ok(false)
}

fn spop2(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if !c.is_relate_to_effect(duel, e) {
        return Ok(());
    }
    duel.special_summon(c, 0, tp, tp, false, false, POS_FACEUP)?;
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_code(duel, EFFECT_CANNOT_SPECIAL_SUMMON);
    e1.set_property(duel, EFFECT_FLAG_PLAYER_TARGET + EFFECT_FLAG_CLIENT_HINT);
    e1.set_description(duel, aux::stringid(ID, 2));
    e1.set_target_range(duel, 1, 0);
    e1.set_target_spsummon(duel, splimit);
    e1.set_reset(duel, RESET_PHASE | PHASE_END);
    duel.register_effect(e1, tp);
    Ok(())
}

pub fn splimit(duel: &Duel, _: EffectId, c: CardId, _: u8, _: u32, _: u8, _: u8, _: Option<EffectId>) -> bool {
    !c.is_set_card(duel, &[SET_PLUNDER_PATROLL])
}
