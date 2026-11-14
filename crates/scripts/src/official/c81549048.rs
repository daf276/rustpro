// Ryu-Ge Rivalry
use super::c52331012::ReturnToHand;
use crate::prelude::*;
use crate::testing::{Check, DECK, SZONE};

const ID: u32 = 81549048;

pub const LISTED_SERIES: &[u16] = &[SET_RYU_GE];

pub const SUPPORT: &[(u32, u16)] = &[(ID, SZONE), (ID, DECK)];
// The Ryu-Ge monsters and Continuous Spells both effects need are batch 735's, not merged yet.
pub const CHECKS: &[Check] = &[
    Check::Report(&Check::ResolvesEffect(aux::stringid(ID, 0))),
    Check::Report(&Check::ResolvesEffect(aux::stringid(ID, 1))),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON + CATEGORY_TOHAND);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_hint_timing(duel, 0, TIMING_STANDBY_PHASE | TIMING_MAIN_END | TIMINGS_CHECK_MONSTER_E);
    e1.set_count_limit(duel, 1, ID);
    e1.set_target(duel, sptg);
    e1.set_operation(duel, spop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_TODECK + CATEGORY_DRAW);
    e2.set_type(duel, EFFECT_TYPE_QUICK_O);
    e2.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e2.set_code(duel, EVENT_FREE_CHAIN);
    e2.set_range(duel, LOCATION_GRAVE);
    e2.set_hint_timing(duel, 0, TIMING_STANDBY_PHASE | TIMING_MAIN_END | TIMINGS_CHECK_MONSTER_E);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_cost(duel, Cost::self_banish);
    e2.set_target(duel, tdtg);
    e2.set_operation(duel, tdop);
    c.register_effect(duel, e2);
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_set_card(duel, &[SET_RYU_GE]) && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let location = LOCATION_HAND | LOCATION_DECK | LOCATION_GRAVE;
    if chk == 0 {
        return Ok(duel.get_location_count(tp, LOCATION_MZONE) > 0
            && duel.is_existing_matching_card(|c, duel| spfilter(c, duel, e, tp), tp, location, 0, 1, None));
    }
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, tp, location as i32);
    duel.set_possible_operation_info(0, CATEGORY_TOHAND, None, 1, tp, LOCATION_MZONE as i32);
    Ok(false)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    if duel.get_location_count(tp, LOCATION_MZONE) == 0 {
        return Ok(());
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
    let filter = aux::necro_valley_filter(|c, duel| spfilter(c, duel, e, tp));
    let g = duel.select_matching_card(tp, filter, tp, LOCATION_HAND | LOCATION_DECK | LOCATION_GRAVE, 0, 1, 1, None)?;
    if let Some(sc) = g.get_first()
        && duel.special_summon(sc, 0, tp, tp, false, false, POS_FACEUP)? > 0
    {
        aux::delayed_operation_full::<ReturnToHand>(duel, sc.into(), PHASE_END, 0, ID, e, tp, aux::stringid(ID, 2), 0);
    }
    Ok(())
}

fn tdfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_RYU_GE]) && c.is_continuous_spell(duel) && c.is_able_to_deck(duel) && c.is_faceup(duel)
}

fn tdtg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    let location = LOCATION_ONFIELD | LOCATION_GRAVE | LOCATION_REMOVED;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, location) && chkc.is_controler(duel, tp) && tdfilter(chkc, duel));
    }
    if chk == 0 {
        return Ok(duel.is_player_can_draw(tp, 1) && duel.is_existing_target(tdfilter, tp, location, 0, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TODECK);
    let g = duel.select_target(tp, tdfilter, tp, location, 0, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_TODECK, g, 1, tp, 0);
    duel.set_operation_info(0, CATEGORY_DRAW, None, 1, tp, 0);
    Ok(false)
}

fn tdop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    let Some(tc) = duel.get_first_target() else { return Ok(()) };
    if tc.is_relate_to_effect(duel, a.e)
        && duel.send_to_deck(tc, PLAYER_NONE, SEQ_DECKBOTTOM, REASON_EFFECT)? > 0
        && tc.is_location(duel, LOCATION_DECK)
        && duel.is_player_can_draw(tp, 0)
    {
        duel.break_effect()?;
        duel.draw(tp, 1, REASON_EFFECT)?;
    }
    Ok(())
}
