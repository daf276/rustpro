// Blessed Winds
use crate::prelude::*;
use crate::testing::{Check, GRAVE, HAND, MZONE, SZONE};

const ID: u32 = 15177750;

// In the hand and the Spell & Trap Zone, with Plant Normal Monsters (Dark Plant, Angel Trumpeter) in the hand and Graveyard and
// Aroma monsters (Angelica, Rosalina) in the Graveyard to summon.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, SZONE),
    (13193642, HAND),
    (13193642, GRAVE),
    (87979586, MZONE),
    (87979586, GRAVE),
    (16759958, GRAVE),
    (24689197, GRAVE),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(0),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
    Check::ResolvesEffect(aux::stringid(ID, 2)),
    Check::ResolvesEffect(aux::stringid(ID, 3)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_RECOVER);
    e2.set_property(duel, EFFECT_FLAG_PLAYER_TARGET);
    e2.set_type(duel, EFFECT_TYPE_QUICK_O);
    e2.set_code(duel, EVENT_FREE_CHAIN);
    e2.set_range(duel, LOCATION_SZONE);
    e2.set_count_limit(duel, 1, ID);
    e2.set_cost(duel, rccost);
    e2.set_target(duel, rctg);
    e2.set_operation(duel, rcop);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 2));
    e3.set_category(duel, CATEGORY_RECOVER + CATEGORY_TODECK);
    e3.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e3.set_type(duel, EFFECT_TYPE_QUICK_O);
    e3.set_code(duel, EVENT_FREE_CHAIN);
    e3.set_range(duel, LOCATION_SZONE);
    e3.set_count_limit(duel, 1, ID);
    e3.set_target(duel, tdtg);
    e3.set_operation(duel, tdop);
    c.register_effect(duel, e3);
    let e4 = duel.create_effect(c);
    e4.set_description(duel, aux::stringid(ID, 3));
    e4.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e4.set_property(duel, EFFECT_FLAG_PLAYER_TARGET);
    e4.set_type(duel, EFFECT_TYPE_QUICK_O);
    e4.set_code(duel, EVENT_FREE_CHAIN);
    e4.set_range(duel, LOCATION_SZONE);
    e4.set_count_limit(duel, 1, ID);
    e4.set_cost(duel, spcost);
    e4.set_target(duel, sptg);
    e4.set_operation(duel, spop);
    c.register_effect(duel, e4);
}

fn tgfilter(c: CardId, duel: &Duel) -> bool {
    c.is_monster(duel)
        && c.is_race(duel, RACE_PLANT)
        && c.is_able_to_grave_as_cost(duel)
        && (c.is_faceup(duel) || c.is_location(duel, LOCATION_HAND))
}

fn rccost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(tgfilter, tp, LOCATION_MZONE | LOCATION_HAND, 0, 1, None));
    }
    duel.hint(HINT_OPSELECTED, 1 - tp, a.e.get_description(duel));
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TOGRAVE);
    let g = duel.select_matching_card(tp, tgfilter, tp, LOCATION_MZONE | LOCATION_HAND, 0, 1, 1, None)?;
    duel.send_to_grave(g, REASON_COST)?;
    Ok(false)
}

fn rctg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(true);
    }
    duel.set_target_player(tp);
    duel.set_target_param(500);
    duel.set_operation_info(0, CATEGORY_RECOVER, None, 0, tp, 500);
    Ok(false)
}

fn rcop(duel: &mut Duel, _: &Args) -> Script<()> {
    let p: u8 = duel.get_chain_info(0, CHAININFO_TARGET_PLAYER);
    let d: u32 = duel.get_chain_info(0, CHAININFO_TARGET_PARAM);
    duel.recover(p, d, REASON_EFFECT)?;
    Ok(())
}

fn tdfilter(c: CardId, duel: &Duel) -> bool {
    c.is_race(duel, RACE_PLANT) && c.is_able_to_deck(duel)
}

fn tdtg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_GRAVE) && chkc.is_controler(duel, tp) && tdfilter(chkc, duel));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(tdfilter, tp, LOCATION_GRAVE, 0, 1, None));
    }
    duel.hint(HINT_OPSELECTED, 1 - tp, a.e.get_description(duel));
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TODECK);
    let g = duel.select_target(tp, tdfilter, tp, LOCATION_GRAVE, 0, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_TODECK, g, 1, 0, 0);
    duel.set_operation_info(0, CATEGORY_RECOVER, None, 1, tp, 500);
    Ok(false)
}

fn tdop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    let tc = duel.get_first_target();
    unported!(duel, tc.is_none());
    if let Some(tc) = tc
        && tc.is_relate_to_effect(duel, a.e)
        && duel.send_to_deck(tc, PLAYER_NONE, SEQ_DECKSHUFFLE, REASON_EFFECT)? > 0
        && tc.is_location(duel, LOCATION_DECK | LOCATION_EXTRA)
    {
        duel.break_effect()?;
        duel.recover(tp, 500, REASON_EFFECT)?;
    }
    Ok(())
}

fn spcost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.check_lp_cost(tp, 1000));
    }
    duel.hint(HINT_OPSELECTED, 1 - tp, a.e.get_description(duel));
    duel.pay_lp_cost(tp, 1000)?;
    Ok(false)
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_set_card(duel, &[SET_AROMA]) && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    stock::special_summon_target(duel, a, chk, |c, duel| spfilter(c, duel, a.e, a.tp), LOCATION_GRAVE)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    if duel.get_location_count(tp, LOCATION_MZONE) <= 0 {
        return Ok(());
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
    let g = duel.select_matching_card(
        tp,
        aux::necro_valley_filter(|c, duel| spfilter(c, duel, e, tp)),
        tp,
        LOCATION_GRAVE,
        0,
        1,
        1,
        None,
    )?;
    if !g.is_empty() {
        duel.special_summon(g, 0, tp, tp, false, false, POS_FACEUP)?;
    }
    Ok(())
}
