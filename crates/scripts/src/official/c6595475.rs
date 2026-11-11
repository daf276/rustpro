// Onomatopaira
use crate::prelude::*;
use crate::testing::{DECK, HAND};

const ID: u32 = 6595475;
const SETCODES: [u16; 4] = [SET_GAGAGA, SET_GOGOGO, SET_DODODO, SET_ZUBABA];

// Onomatokage and Dodododwarf Gogogoglove in the Deck to search, and a card to discard.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (55088578, DECK),
    (55088578, DECK),
    (59724555, DECK),
    (59724555, DECK),
    (59724555, HAND),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_category(duel, CATEGORY_TOHAND + CATEGORY_SEARCH);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_count_limit_full(duel, 1, ID, EFFECT_COUNT_CODE_OATH);
    e1.set_cost(duel, cost);
    e1.set_target(duel, target);
    e1.set_operation(duel, activate);
    c.register_effect(duel, e1);
}

fn cost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(
            CardId::is_able_to_grave_as_cost,
            tp,
            LOCATION_HAND,
            0,
            1,
            Some(a.e.get_handler(duel).into()),
        ));
    }
    duel.discard_hand(tp, CardId::is_able_to_grave_as_cost, 1, 1, REASON_COST)?;
    Ok(false)
}

fn filter(c: CardId, duel: &Duel) -> bool {
    c.is_monster(duel) && SETCODES.iter().any(|&setcode| c.is_set_card(duel, &[setcode])) && c.is_able_to_hand(duel)
}

fn target(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    stock::exists_target(duel, a, chk, filter, LOCATION_DECK, CATEGORY_TOHAND)
}

fn rescon(duel: &Duel, sg: CardSet, _: CardSet, _: Option<CardId>) -> (bool, bool) {
    let count = SETCODES
        .iter()
        .filter(|&&setcode| sg.is_exists(duel, |c, duel| c.is_set_card(duel, &[setcode]), 1, None))
        .count();
    (count >= sg.len(), false)
}

fn activate(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    let g = duel.get_matching_group(filter, tp, LOCATION_DECK, 0, None);
    if g.is_empty() {
        return Ok(());
    }
    let tg = aux::select_unselect_group(duel, g, 1, 2, &rescon, tp, HINTMSG_ATOHAND)?;
    if !tg.is_empty() {
        duel.send_to_hand(tg, PLAYER_NONE, REASON_EFFECT)?;
        duel.confirm_cards(1 - tp, tg)?;
    }
    Ok(())
}
