// Tearlaments Kashtira
use crate::prelude::*;
use crate::testing::{Check, DECK, GRAVE, HAND};

const ID: u32 = 4928565;

// In the hand with Merrli in the hand and Graveyard to banish, and in the Deck for the mill of Merrli to send.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, DECK),
    (ID, DECK),
    (74078255, HAND),
    (74078255, GRAVE),
    (74078255, DECK),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
    Check::ResolvesEffect(aux::stringid(ID, 2)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON + CATEGORY_REMOVE);
    e1.set_type(duel, EFFECT_TYPE_QUICK_O);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_range(duel, LOCATION_HAND);
    e1.set_count_limit(duel, 1, ID);
    e1.set_hint_timing(duel, 0, TIMINGS_CHECK_MONSTER | TIMING_MAIN_END);
    e1.set_condition(duel, |duel, _| duel.is_main_phase());
    e1.set_target(duel, sptg);
    e1.set_operation(duel, spop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_DECKDES);
    e2.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e2.set_property(duel, EFFECT_FLAG_DELAY);
    e2.set_code(duel, EVENT_SUMMON_SUCCESS);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_target(duel, smltg);
    e2.set_operation(duel, smlop);
    c.register_effect(duel, e2);
    let e3 = e2.clone_effect(duel);
    e3.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    c.register_effect(duel, e3);
    let e4 = e2.clone_effect(duel);
    e4.set_description(duel, aux::stringid(ID, 2));
    e4.set_code(duel, EVENT_TO_GRAVE);
    e4.set_count_limit(duel, 1, (ID, 2));
    e4.set_condition(duel, |duel, a| a.e.get_handler(duel).is_reason(duel, REASON_EFFECT));
    e4.set_target(duel, tgmltg);
    e4.set_operation(duel, tgmlop);
    c.register_effect(duel, e4);
}

fn rmfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_TEARLAMENTS, SET_KASHTIRA])
        && c.is_able_to_remove(duel)
        && (c.is_location(duel, LOCATION_HAND) || aux::sp_elim_filter(c, duel, true, false))
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    let locations = LOCATION_HAND | LOCATION_GRAVE | LOCATION_MZONE;
    if chk == 0 {
        return Ok(duel.get_location_count(tp, LOCATION_MZONE) > 0
            && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
            && duel.is_existing_matching_card(rmfilter, tp, locations, 0, 1, Some(c.into())));
    }
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, Some(c.into()), 1, 0, 0);
    duel.set_operation_info(0, CATEGORY_REMOVE, None, 1, tp, locations as i32);
    Ok(false)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if !c.is_relate_to_effect(duel, e) || duel.special_summon(c, 0, tp, tp, false, false, POS_FACEUP)? == 0 {
        return Ok(());
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_REMOVE);
    let g = duel.select_matching_card(tp, rmfilter, tp, LOCATION_HAND | LOCATION_GRAVE | LOCATION_MZONE, 0, 1, 1, None)?;
    if !g.is_empty() {
        duel.remove(g, POS_FACEUP, REASON_EFFECT)?;
    }
    Ok(())
}

fn smltg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.is_player_can_discard_deck(tp, 3) || duel.is_player_can_discard_deck(1 - tp, 3));
    }
    duel.set_operation_info(0, CATEGORY_DECKDES, None, 0, PLAYER_EITHER, 3);
    Ok(false)
}

fn smlop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    let b1 = duel.is_player_can_discard_deck(tp, 3);
    let b2 = duel.is_player_can_discard_deck(1 - tp, 3);
    let op = duel.select_effect(tp, &[(b1, aux::stringid(ID, 3)), (b2, aux::stringid(ID, 4))])?;
    if let Some(op) = op {
        duel.discard_deck(if op == 1 { tp } else { 1 - tp }, 3, REASON_EFFECT)?;
    }
    Ok(())
}

fn tgmltg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.is_player_can_discard_deck(tp, 2));
    }
    duel.set_operation_info(0, CATEGORY_DECKDES, None, 0, tp, 2);
    Ok(false)
}

fn tgmlop(duel: &mut Duel, a: &Args) -> Script<()> {
    duel.discard_deck(a.tp, 2, REASON_EFFECT)?;
    Ok(())
}
