// Enato Enneacraft - "oknirIA"
use super::c28454232::{flip_quick, register_counter, register_set_summon};
use super::c92171126::{register_reveal, thop, thtg};
use crate::prelude::*;
use crate::testing::{Check, DECK, GRAVE, HAND, MZONE, OPPONENT, PZONE};

const ID: u32 = 55965529;

pub const COUNTER_PLACE_LIST: &[u16] = &[COUNTER_EC];
pub const LISTED_SERIES: &[u16] = &[SET_ENNEACRAFT];

// In the hand, a Pendulum Zone, face-down on the field and in the Deck to reveal and search, with the opponent's Glow-Up
// Bulb activating in their GY and Sangan there to shuffle.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, PZONE),
    (ID, MZONE),
    (ID, MZONE),
    (ID, DECK),
    (ID, DECK),
    (ID, DECK),
    (ID, DECK),
    (67441435, GRAVE | OPPONENT),
    (26202165, GRAVE | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
    Check::ResolvesEffect(aux::stringid(ID, 2)),
    Check::ResolvesEffect(aux::stringid(ID, 3)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    register_counter(duel, c);
    register_reveal::<ID>(duel, c);
    register_set_summon::<ID>(duel, c);
    let e4 = flip_quick::<ID>(duel, c, CATEGORY_TOHAND + CATEGORY_SEARCH, 0);
    e4.set_condition(duel, |duel, a| {
        let loc: u32 = duel.get_chain_info(a.ev as u8, CHAININFO_TRIGGERING_LOCATION);
        a.e.get_handler(duel).is_facedown(duel) && a.ep == 1 - a.tp && loc & (LOCATION_GRAVE | LOCATION_REMOVED) > 0
    });
    e4.set_target(duel, thtg);
    e4.set_operation(duel, thop);
    c.register_effect(duel, e4);
    let e5 = duel.create_effect(c);
    e5.set_description(duel, aux::stringid(ID, 3));
    e5.set_category(duel, CATEGORY_TODECK);
    e5.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_F);
    e5.set_code(duel, EVENT_FLIP);
    e5.set_target(duel, tdtg);
    e5.set_operation(duel, tdop);
    c.register_effect(duel, e5);
}

fn tdtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(true);
    }
    duel.set_operation_info(0, CATEGORY_TODECK, None, 1, 1 - a.tp, (LOCATION_GRAVE | LOCATION_REMOVED) as i32);
    Ok(false)
}

fn tdop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TODECK);
    let filter = aux::necro_valley_filter(CardId::is_able_to_deck);
    let g = duel.select_matching_card(tp, filter, tp, 0, LOCATION_GRAVE | LOCATION_REMOVED, 1, 3, None)?;
    if !g.is_empty() {
        duel.hint_selection(g);
        duel.send_to_deck(g, PLAYER_NONE, SEQ_DECKSHUFFLE, REASON_EFFECT)?;
    }
    Ok(())
}
