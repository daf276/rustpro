// Chaos Space
use crate::prelude::*;
use crate::testing::{Check, DECK, GRAVE, HAND, REMOVED};

const ID: u32 = 99266988;
const LIGHT_DARK: u32 = ATTRIBUTE_DARK + ATTRIBUTE_LIGHT;

// Blue-Eyes White Dragon to discard, Chaos Sorcerer and Black Luster Soldier - Envoy of the Beginning (DARK and LIGHT)
// to search, and a copy in the Graveyard with both banished to return one.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, GRAVE),
    (CARD_BLUEEYES_W_DRAGON, HAND),
    (9596126, DECK),
    (9596126, DECK),
    (72989439, DECK),
    (9596126, REMOVED),
    (72989439, REMOVED),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_SEARCH + CATEGORY_TOHAND);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_count_limit(duel, 1, ID);
    e1.set_cost(duel, thcost);
    e1.set_target(duel, thtg);
    e1.set_operation(duel, thop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_TODECK + CATEGORY_DRAW);
    e2.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e2.set_type(duel, EFFECT_TYPE_IGNITION);
    e2.set_range(duel, LOCATION_GRAVE);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_cost(duel, Cost::self_banish);
    e2.set_target(duel, tdtg);
    e2.set_operation(duel, tdop);
    c.register_effect(duel, e2);
}

fn thfilter1(c: CardId, duel: &Duel, att: u32) -> bool {
    c.is_able_to_hand(duel)
        && c.is_monster(duel)
        && c.is_attribute(duel, LIGHT_DARK)
        && c.is_level_above(duel, 4)
        && c.is_level_below(duel, 8)
        && !c.is_summonable_card(duel)
        && !c.is_attribute(duel, att)
}

fn tgfilter(c: CardId, duel: &Duel, tp: u8) -> bool {
    c.is_monster(duel)
        && c.is_attribute(duel, LIGHT_DARK)
        && c.is_able_to_grave_as_cost(duel)
        && duel.is_existing_matching_card(|tc, duel| thfilter1(tc, duel, c.get_attribute(duel)), tp, LOCATION_DECK, 0, 1, None)
}

fn thcost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(|c, duel| tgfilter(c, duel, tp), tp, LOCATION_HAND, 0, 1, None));
    }
    duel.discard_hand(tp, |c, duel| tgfilter(c, duel, tp), 1, 1, REASON_COST)?;
    let att = duel.get_operated_group().get_first().map_or(0, |c| c.get_attribute(duel));
    a.e.set_label(duel, i64::from(att));
    Ok(false)
}

fn thfilter2(c: CardId, duel: &Duel) -> bool {
    c.is_attribute(duel, LIGHT_DARK)
        && c.is_level_above(duel, 4)
        && c.is_level_below(duel, 8)
        && !c.is_summonable_card(duel)
        && c.is_able_to_hand(duel)
}

fn thtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    stock::exists_target(duel, a, chk, thfilter2, LOCATION_DECK, CATEGORY_TOHAND)
}

fn thop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    let att = a.e.get_label(duel) as u32;
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_ATOHAND);
    let g1 = duel.select_matching_card(tp, |c, duel| thfilter1(c, duel, att), tp, LOCATION_DECK, 0, 1, 1, None)?;
    if !g1.is_empty() {
        duel.send_to_hand(g1, PLAYER_NONE, REASON_EFFECT)?;
        duel.confirm_cards(1 - tp, g1)?;
    }
    Ok(())
}

fn tdfilter(c: CardId, duel: &Duel) -> bool {
    c.is_faceup(duel) && c.is_attribute(duel, LIGHT_DARK) && c.is_able_to_deck(duel) && !c.is_summonable_card(duel)
}

fn tdtg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    // The script calls the undefined s.filter for chkc, so the target check fails with a Lua error.
    if chkc.is_some() {
        return Ok(false);
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(tdfilter, tp, LOCATION_REMOVED, 0, 1, None) && duel.is_player_can_draw(tp, 1));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TODECK);
    let g = duel.select_target(tp, tdfilter, tp, LOCATION_REMOVED, 0, 1, 1, None)?;
    duel.set_target_player(tp);
    duel.set_target_param(1);
    duel.set_operation_info(0, CATEGORY_TOHAND, g, 1, 0, 0);
    duel.set_operation_info(0, CATEGORY_DRAW, None, 0, tp, 3);
    Ok(false)
}

fn tdop(duel: &mut Duel, _: &Args) -> Script<()> {
    let tc = duel.get_first_target();
    let p: u8 = duel.get_chain_info(0, CHAININFO_TARGET_PLAYER);
    let d: u32 = duel.get_chain_info(0, CHAININFO_TARGET_PARAM);
    if let Some(tc) = tc
        && duel.send_to_deck(tc, PLAYER_NONE, SEQ_DECKBOTTOM, REASON_EFFECT)? == 1
    {
        duel.break_effect()?;
        duel.draw(p, d, REASON_EFFECT)?;
    }
    Ok(())
}
