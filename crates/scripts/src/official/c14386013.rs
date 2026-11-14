// Dark Spirit's Mastery
use crate::prelude::*;
use crate::testing::{Check, DECK, HAND};

const ID: u32 = 14386013;

pub const LISTED_SERIES: &[u16] = &[SET_SPIRIT_MESSAGE];
pub const LISTED_NAMES: &[u32] = &[CARD_DESTINY_BOARD];
// Dark Necrofear (Level 8 Fiend) in the Deck to search and Ojama Green to discard. The GY effect needs Destiny Board or a
// "Spirit Message", which are unported.
pub const SUPPORT: &[(u32, u16)] = &[(ID, HAND), (12482652, HAND), (31829185, DECK), (31829185, DECK), (31829185, DECK)];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_TOHAND + CATEGORY_SEARCH);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_count_limit(duel, 1, ID);
    e1.set_cost(duel, cost);
    e1.set_target(duel, target);
    e1.set_operation(duel, activate);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_TODECK + CATEGORY_DRAW);
    e2.set_type(duel, EFFECT_TYPE_IGNITION);
    e2.set_range(duel, LOCATION_GRAVE);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_condition(duel, aux::exccon);
    e2.set_cost(duel, Cost::self_banish);
    e2.set_target(duel, tdrtg);
    e2.set_operation(duel, tdrop);
    c.register_effect(duel, e2);
}

fn cost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        let c = a.e.get_handler(duel);
        return Ok(duel.is_existing_matching_card(CardId::is_discardable, tp, LOCATION_HAND, 0, 1, Some(c.into())));
    }
    duel.discard_hand(tp, CardId::is_discardable, 1, 1, REASON_COST | REASON_DISCARD)?;
    Ok(false)
}

fn thfilter(c: CardId, duel: &Duel) -> bool {
    (c.is_code(duel, CARD_DESTINY_BOARD) || c.is_race(duel, RACE_FIEND) && c.get_level(duel) == 8) && c.is_able_to_hand(duel)
}

fn target(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    stock::exists_target(duel, a, chk, thfilter, LOCATION_DECK | LOCATION_GRAVE, CATEGORY_TOHAND)
}

fn activate(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_ATOHAND);
    let g = duel.select_matching_card(
        tp,
        aux::necro_valley_filter(thfilter),
        tp,
        LOCATION_DECK | LOCATION_GRAVE,
        0,
        1,
        1,
        None,
    )?;
    if !g.is_empty() {
        duel.send_to_hand(g, PLAYER_NONE, REASON_EFFECT)?;
        duel.confirm_cards(1 - tp, g)?;
    }
    Ok(())
}

fn tdfilter(c: CardId, duel: &Duel) -> bool {
    (c.is_code(duel, CARD_DESTINY_BOARD) || c.is_set_card(duel, &[SET_SPIRIT_MESSAGE])) && c.is_able_to_deck(duel)
}

fn tdrtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    let g = duel.get_matching_group(tdfilter, tp, LOCATION_HAND | LOCATION_GRAVE, 0, None);
    if chk == 0 {
        let classes = g.get_class_count(duel, |c, duel| i64::from(c.get_code(duel)));
        return Ok(duel.is_player_can_draw(tp, 0) && aux::select_unselect_group_check(duel, g, 1, classes, &aux::dncheck));
    }
    duel.set_operation_info(0, CATEGORY_TODECK, None, 1, tp, (LOCATION_HAND | LOCATION_GRAVE) as i32);
    duel.set_operation_info(0, CATEGORY_DRAW, None, 0, tp, 1);
    Ok(false)
}

fn tdrop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    let g = duel.get_matching_group(aux::necro_valley_filter(tdfilter), tp, LOCATION_HAND | LOCATION_GRAVE, 0, None);
    if g.is_empty() {
        return Ok(());
    }
    let classes = g.get_class_count(duel, |c, duel| i64::from(c.get_code(duel)));
    // The Lua passes a nil "rescon", which accepts any group.
    let sg = aux::select_unselect_group(duel, g, 1, classes, &|_, _, _, _| (true, false), tp, HINTMSG_TODECK)?;
    duel.confirm_cards(1 - tp, sg)?;
    duel.send_to_deck(sg, PLAYER_NONE, SEQ_DECKBOTTOM, REASON_EFFECT)?;
    let og = duel.get_operated_group();
    let ct = og.filter_count(duel, |c, duel| c.is_location(duel, LOCATION_DECK), None);
    if ct > 0 {
        duel.sort_deckbottom(tp, tp, ct as u16)?;
        duel.break_effect()?;
        duel.draw(tp, ct as u32, REASON_EFFECT)?;
    }
    Ok(())
}
