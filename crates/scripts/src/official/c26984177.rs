// Walls of the Imperial Tomb
use crate::prelude::*;
use crate::testing::{Check, DECK, HAND, MZONE};

const ID: u32 = 26984177;
const HORUS_THE_BLACK_FLAME_DEITY: u32 = 99307040;

pub const LISTED_NAMES: &[u32] = &[CARD_KING_SARCOPHAGUS, HORUS_THE_BLACK_FLAME_DEITY];

// In the hand to activate, with Horus the Black Flame Deity in the hand to activate for the draw, Imsety on the field for it
// and in the Deck to search.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, HAND),
    (HORUS_THE_BLACK_FLAME_DEITY, HAND),
    (HORUS_THE_BLACK_FLAME_DEITY, DECK),
    (84941194, MZONE),
    (84941194, DECK),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_SINGLE);
    e2.set_property(duel, EFFECT_FLAG_SINGLE_RANGE);
    e2.set_code(duel, EFFECT_CHANGE_CODE);
    e2.set_range(duel, LOCATION_FZONE);
    e2.set_value(duel, CARD_KING_SARCOPHAGUS as i32);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 0));
    e3.set_category(duel, CATEGORY_TOHAND + CATEGORY_SEARCH + CATEGORY_TODECK);
    e3.set_type(duel, EFFECT_TYPE_IGNITION);
    e3.set_range(duel, LOCATION_FZONE);
    e3.set_count_limit(duel, 1, ID);
    e3.set_target(duel, thtg);
    e3.set_operation(duel, thop);
    c.register_effect(duel, e3);
    let e4 = duel.create_effect(c);
    e4.set_description(duel, aux::stringid(ID, 1));
    e4.set_category(duel, CATEGORY_DRAW);
    e4.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e4.set_property(duel, EFFECT_FLAG_DELAY);
    e4.set_code(duel, EVENT_CHAINING);
    e4.set_range(duel, LOCATION_FZONE);
    e4.set_condition(duel, drcon);
    e4.set_target(duel, stock::draw_target::<1>);
    e4.set_operation(duel, stock::draw);
    c.register_effect(duel, e4);
}

fn thfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_HORUS]) && c.is_monster(duel) && c.is_able_to_hand(duel)
}

fn thtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(thfilter, tp, LOCATION_DECK, 0, 1, None));
    }
    duel.set_operation_info(0, CATEGORY_TOHAND, None, 1, tp, LOCATION_DECK as i32);
    duel.set_operation_info(0, CATEGORY_TODECK, None, 1, tp, LOCATION_HAND as i32);
    Ok(false)
}

fn thop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_ATOHAND);
    let hg = duel.select_matching_card(tp, thfilter, tp, LOCATION_DECK, 0, 1, 1, None)?;
    if hg.is_empty() || duel.send_to_hand(hg, PLAYER_NONE, REASON_EFFECT)? == 0 {
        return Ok(());
    }
    duel.confirm_cards(1 - tp, hg)?;
    duel.shuffle_hand(tp)?;
    duel.shuffle_deck(tp)?;
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TODECK);
    let dg = duel.select_matching_card(tp, CardId::is_able_to_deck, tp, LOCATION_HAND, 0, 1, 1, None)?;
    if !dg.is_empty() {
        duel.break_effect()?;
        duel.send_to_deck(dg, PLAYER_NONE, SEQ_DECKBOTTOM, REASON_EFFECT)?;
    }
    Ok(())
}

fn drcon(duel: &Duel, a: &Args) -> bool {
    let code1: u32 = duel.get_chain_info(a.ev as u8, CHAININFO_TRIGGERING_CODE);
    let code2: u32 = duel.get_chain_info(a.ev as u8, CHAININFO_TRIGGERING_CODE2);
    a.ep == a.tp && (code1 == HORUS_THE_BLACK_FLAME_DEITY || code2 == HORUS_THE_BLACK_FLAME_DEITY)
}
