// Floowandereeze and the Advent of Adventure
use crate::prelude::*;
use crate::testing::{DECK, HAND};

const ID: u32 = 69087397;

pub const LISTED_SERIES: &[u16] = &[SET_FLOOWANDEREEZE];

// Copies in hand, Winged Beasts in hand to banish, and Floowandereeze monsters in the Deck to add.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (76812113, HAND),
    (18940725, HAND),
    (18940725, DECK),
    (54334420, DECK),
    (80433039, DECK),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_category(duel, CATEGORY_TOHAND + CATEGORY_SEARCH + CATEGORY_RECOVER);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_count_limit_full(duel, 1, ID, EFFECT_COUNT_CODE_OATH);
    e1.set_hint_timing(duel, 0, TIMING_END_PHASE);
    e1.set_cost(duel, cost);
    e1.set_target(duel, target);
    e1.set_operation(duel, activate);
    c.register_effect(duel, e1);
}

fn filter(c: CardId, duel: &Duel) -> bool {
    c.is_monster(duel)
        && c.is_race(duel, RACE_WINGEDBEAST)
        && c.is_able_to_remove_as_cost(duel)
        && (c.is_faceup(duel) || !c.is_on_field(duel))
}

fn cost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    stock::banish_cost(duel, a, chk, filter, LOCATION_HAND | LOCATION_MZONE)
}

fn filteradd(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_FLOOWANDEREEZE]) && (c.is_monster(duel) || c.is_type(duel, TYPE_FIELD)) && c.is_able_to_hand(duel)
}

fn target(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    stock::exists_target(duel, a, chk, filteradd, LOCATION_DECK, CATEGORY_TOHAND)
}

fn activate(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_ATOHAND);
    let g = duel.select_matching_card(tp, filteradd, tp, LOCATION_DECK, 0, 1, 1, None)?;
    if g.is_empty() || duel.send_to_hand(g, PLAYER_NONE, REASON_EFFECT)? == 0 {
        return Ok(());
    }
    duel.confirm_cards(1 - tp, g)?;
    duel.break_effect()?;
    duel.recover(tp, 500, REASON_EFFECT)?;
    Ok(())
}
