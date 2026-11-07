// Elemental HERO Shadow Mist
use crate::prelude::*;
use crate::testing::{Check, DECK, GRAVE, HAND};

const ID: u32 = 50720316;

pub const LISTED_NAMES: &[u32] = &[ID];

// In the GY to be revived by Monster Reborn and a second copy in the Deck to send with Foolish Burial, with Mask Change
// (a Quick-Play Spell) and Elemental HERO Stratos in the Deck to search.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, GRAVE),
    (ID, DECK),
    (83764718, HAND),
    (81439173, HAND),
    (21143940, DECK),
    (40044918, DECK),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_TOHAND + CATEGORY_SEARCH);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e1.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e1.set_property(duel, EFFECT_FLAG_DAMAGE_STEP + EFFECT_FLAG_DELAY);
    e1.set_count_limit(duel, 1, ID);
    e1.set_cost(duel, cost);
    e1.set_target(duel, thtg1);
    e1.set_operation(duel, tgop1);
    c.register_effect(duel, e1);
    let e2 = e1.clone_effect(duel);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_code(duel, EVENT_TO_GRAVE);
    e2.set_target(duel, thtg2);
    e2.set_operation(duel, tgop2);
    c.register_effect(duel, e2);
}

fn cost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    if chk == 0 {
        return Ok(true);
    }
    duel.hint(HINT_OPSELECTED, 1 - a.tp, a.e.get_description(duel));
    Ok(false)
}

fn thfilter1(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_CHANGE]) && c.is_type(duel, TYPE_QUICKPLAY) && c.is_able_to_hand(duel)
}

fn thfilter2(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_HERO]) && c.is_monster(duel) && !c.is_code(duel, ID) && c.is_able_to_hand(duel)
}

fn thtg(duel: &mut Duel, tp: u8, chk: i32, filter: fn(CardId, &Duel) -> bool) -> Script<bool> {
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(filter, tp, LOCATION_DECK, 0, 1, None));
    }
    duel.set_operation_info(0, CATEGORY_TOHAND, None, 1, tp, LOCATION_DECK as i32);
    Ok(false)
}

fn tgop(duel: &mut Duel, tp: u8, filter: fn(CardId, &Duel) -> bool) -> Script<()> {
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_ATOHAND);
    let g = duel.select_matching_card(tp, filter, tp, LOCATION_DECK, 0, 1, 1, None)?;
    if !g.is_empty() {
        duel.send_to_hand(g, PLAYER_NONE, REASON_EFFECT)?;
        duel.confirm_cards(1 - tp, g)?;
    }
    Ok(())
}

fn thtg1(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    thtg(duel, a.tp, chk, thfilter1)
}

fn tgop1(duel: &mut Duel, a: &Args) -> Script<()> {
    tgop(duel, a.tp, thfilter1)
}

fn thtg2(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    thtg(duel, a.tp, chk, thfilter2)
}

fn tgop2(duel: &mut Duel, a: &Args) -> Script<()> {
    tgop(duel, a.tp, thfilter2)
}
