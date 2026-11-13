// Advent of Atlantis
use crate::prelude::*;
use crate::testing::{Check, DECK, HAND, SZONE};

const ID: u32 = 1280391;

pub const LISTED_NAMES: &[u32] = &[CARD_UMI, CARD_ATLANTIS_CITY_OF_THE_SEA_DRAGON];

// Copies on the field and in the hand, with Umi and Level 7 Sea Serpents (Ignihorse Dragonis) in the Deck, and Mystical Space
// Typhoon in the hand to send it to the Graveyard.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, SZONE),
    (ID, HAND),
    (CARD_UMI, DECK),
    (CARD_UMI, DECK),
    (28103028, DECK),
    (28103028, DECK),
    (28103028, DECK),
    (5318639, HAND),
    (5318639, HAND),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e0 = duel.create_effect(c);
    e0.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e0.set_code(duel, EVENT_FREE_CHAIN);
    c.register_effect(duel, e0);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_property(duel, EFFECT_FLAG_SINGLE_RANGE);
    e1.set_code(duel, EFFECT_CHANGE_CODE);
    e1.set_range(duel, LOCATION_SZONE);
    e1.set_value(duel, CARD_UMI as i32);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 0));
    e2.set_type(duel, EFFECT_TYPE_IGNITION);
    e2.set_range(duel, LOCATION_SZONE);
    e2.set_count_limit(duel, 1, (ID, 0));
    e2.set_target(duel, pltg);
    e2.set_operation(duel, plop);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 1));
    e3.set_category(duel, CATEGORY_TOHAND + CATEGORY_SEARCH);
    e3.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e3.set_property(duel, EFFECT_FLAG_DELAY);
    e3.set_code(duel, EVENT_TO_GRAVE);
    e3.set_count_limit(duel, 1, (ID, 1));
    e3.set_target(duel, thtg);
    e3.set_operation(duel, thop);
    c.register_effect(duel, e3);
}

fn plfilter(c: CardId, duel: &Duel, tp: u8) -> bool {
    (c.is_code(duel, CARD_ATLANTIS_CITY_OF_THE_SEA_DRAGON) || c.is_code(duel, CARD_UMI))
        && c.is_field_spell(duel)
        && !c.is_forbidden(duel)
        && c.check_unique_on_field(duel, tp)
}

fn pltg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    Ok(chk == 0 && duel.is_existing_matching_card(|c, duel| plfilter(c, duel, tp), tp, LOCATION_DECK, 0, 1, None))
}

fn plop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TOFIELD);
    let g = duel.select_matching_card(tp, |c, duel| plfilter(c, duel, tp), tp, LOCATION_DECK, 0, 1, 1, None)?;
    if let Some(sc) = g.get_first() {
        duel.place_field_spell(sc, tp)?;
    }
    Ok(())
}

fn thfilter(c: CardId, duel: &Duel) -> bool {
    c.is_level(duel, &[7]) && c.is_race(duel, RACE_SEASERPENT) && c.is_able_to_hand(duel)
}

fn thtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    stock::exists_target(duel, a, chk, thfilter, LOCATION_DECK, CATEGORY_TOHAND)
}

fn thop(duel: &mut Duel, a: &Args) -> Script<()> {
    stock::search(duel, a, thfilter, LOCATION_DECK)
}
