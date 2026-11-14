// Fallen Angel in Darkness
use crate::prelude::*;
use crate::testing::{Check, DECK, HAND, SZONE};

const ID: u32 = 93023136;

pub const LISTED_SERIES: &[u16] = &[SET_DARKLORD];

const DARKLORD_GULGOLET: u32 = 84031359;
const DARKLORD_DJEHUTY: u32 = 10426067;

// Set, with Fairies in the hand to send and Darklords in the Deck to Special Summon.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, SZONE),
    (DARKLORD_DJEHUTY, HAND),
    (DARKLORD_DJEHUTY, HAND),
    (DARKLORD_GULGOLET, DECK),
    (DARKLORD_GULGOLET, DECK),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_count_limit_full(duel, 1, ID, EFFECT_COUNT_CODE_OATH);
    e1.set_cost(duel, cost);
    e1.set_target(duel, target);
    e1.set_operation(duel, activate);
    e1.set_hint_timing(duel, 0, TIMING_STANDBY_PHASE | TIMING_MAIN_END | TIMINGS_CHECK_MONSTER_E);
    c.register_effect(duel, e1);
}

fn costfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_race(duel, RACE_FAIRY)
        && duel.get_mzone_count(tp, Some(c.into())) > 0
        && (c.is_location(duel, LOCATION_HAND) || c.is_faceup(duel))
        && c.is_able_to_grave_as_cost(duel)
        && duel.is_existing_matching_card(
            |sc, duel| spfilter(sc, duel, e, tp),
            tp,
            LOCATION_HAND | LOCATION_DECK | LOCATION_REMOVED,
            0,
            1,
            Some(c.into()),
        )
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_set_card(duel, &[SET_DARKLORD])
        && c.is_monster(duel)
        && (c.is_location(duel, LOCATION_HAND | LOCATION_DECK) || c.is_faceup(duel))
        && c.is_can_be_special_summoned(duel, e, 0, tp, true, false)
}

fn cost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    e.set_label(duel, -100);
    let filter = |c, duel: &Duel| costfilter(c, duel, e, tp);
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(filter, tp, LOCATION_HAND | LOCATION_MZONE, 0, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TOGRAVE);
    let g = duel.select_matching_card(tp, filter, tp, LOCATION_HAND | LOCATION_MZONE, 0, 1, 1, None)?;
    duel.send_to_grave(g, REASON_COST)?;
    Ok(false)
}

fn target(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if chk == 0 {
        let mmz_chk = e.get_label(duel) == -100 || duel.get_location_count(tp, LOCATION_MZONE) > 0;
        e.set_label(duel, 0);
        let filter = |c, duel: &Duel| spfilter(c, duel, e, tp);
        return Ok(mmz_chk && duel.is_existing_matching_card(filter, tp, LOCATION_HAND | LOCATION_DECK | LOCATION_REMOVED, 0, 1, None));
    }
    duel.set_operation_info(
        0,
        CATEGORY_SPECIAL_SUMMON,
        None,
        1,
        tp,
        (LOCATION_HAND | LOCATION_DECK | LOCATION_REMOVED) as i32,
    );
    Ok(false)
}

fn activate(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    if duel.get_location_count(tp, LOCATION_MZONE) <= 0 {
        return Ok(());
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
    let filter = |c, duel: &Duel| spfilter(c, duel, e, tp);
    let g = duel.select_matching_card(tp, filter, tp, LOCATION_HAND | LOCATION_DECK | LOCATION_REMOVED, 0, 1, 1, None)?;
    if !g.is_empty() {
        duel.special_summon(g, 0, tp, tp, true, false, POS_FACEUP)?;
    }
    Ok(())
}
