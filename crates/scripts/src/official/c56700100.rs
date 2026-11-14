// My Friend Purrely
use crate::prelude::*;
use crate::testing::{Check, DECK, GRAVE, HAND, MZONE, OPPONENT, SZONE};

const ID: u32 = 56700100;

pub const LISTED_NAMES: &[u32] = &[ID];
pub const LISTED_SERIES: &[u16] = &[SET_PURRELY];

// On the field with Purrely Quick-Play Spells in the Deck and Graveyard, and an Epurrely Plump for the opponent's Raigeki.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, SZONE),
    (21347668, DECK),
    (29599813, DECK),
    (55584558, DECK),
    (82105704, DECK),
    (82105704, GRAVE),
    (24434049, MZONE),
    (24434049, MZONE),
    (12580477, HAND | OPPONENT),
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
    e2.set_description(duel, aux::stringid(ID, 0));
    e2.set_category(duel, CATEGORY_TOHAND + CATEGORY_SEARCH);
    e2.set_type(duel, EFFECT_TYPE_IGNITION);
    e2.set_range(duel, LOCATION_SZONE);
    e2.set_count_limit(duel, 1, ID);
    e2.set_cost(duel, Cost::pay_lp::<500>);
    e2.set_target(duel, thtg);
    e2.set_operation(duel, thop);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 1));
    e3.set_category(duel, CATEGORY_TOHAND);
    e3.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e3.set_property(duel, EFFECT_FLAG_DELAY + EFFECT_FLAG_DAMAGE_STEP);
    e3.set_code(duel, EVENT_LEAVE_FIELD);
    e3.set_range(duel, LOCATION_SZONE);
    e3.set_count_limit(duel, 1, (ID, 1));
    e3.set_condition(duel, rthcond);
    e3.set_target(duel, rthtg);
    e3.set_operation(duel, rthop);
    c.register_effect(duel, e3);
}

fn thfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_PURRELY]) && c.is_able_to_hand(duel) && !c.is_code(duel, ID)
}

fn thtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(thfilter, tp, LOCATION_DECK, 0, 3, None));
    }
    duel.set_operation_info(0, CATEGORY_TOHAND, None, 1, tp, LOCATION_DECK as i32);
    Ok(false)
}

fn thop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    let g = duel.get_matching_group(thfilter, tp, LOCATION_DECK, 0, None);
    if g.len() >= 3 {
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_ATOHAND);
        let sg = g.select(duel, tp, 3, 3, None)?;
        duel.confirm_cards(1 - tp, sg)?;
        duel.shuffle_deck(tp)?;
        let tg = sg.random_select(duel, 1 - tp, 1);
        duel.send_to_hand(tg, PLAYER_NONE, REASON_EFFECT)?;
        duel.shuffle_deck(tp)?;
    }
    Ok(())
}

fn xyzfilter(c: CardId, duel: &Duel, tp: u8) -> bool {
    c.is_previous_position(duel, POS_FACEUP)
        && c.is_type(duel, TYPE_XYZ)
        && c.is_previous_set_card(duel, &[SET_PURRELY])
        && c.is_previous_controler(duel, tp)
        && c.get_reason_player(duel) == 1 - tp
}

fn rthcond(duel: &Duel, a: &Args) -> bool {
    let tp = a.tp;
    a.eg.unwrap_or_default().is_exists(duel, |c, duel| xyzfilter(c, duel, tp), 1, None)
}

fn qpfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_PURRELY]) && c.is_spell(duel) && c.is_type(duel, TYPE_QUICKPLAY) && c.is_able_to_hand(duel)
}

fn rthtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    stock::exists_target(duel, a, chk, qpfilter, LOCATION_GRAVE, CATEGORY_TOHAND)
}

fn rthop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_ATOHAND);
    let g = duel.get_matching_group(qpfilter, tp, LOCATION_GRAVE, 0, None);
    let sg = aux::select_unselect_group(duel, g, 1, 3, &aux::dncheck, tp, HINTMSG_ATOHAND)?;
    if !sg.is_empty() {
        duel.send_to_hand(sg, PLAYER_NONE, REASON_EFFECT)?;
        duel.confirm_cards(1 - tp, sg)?;
    }
    Ok(())
}
