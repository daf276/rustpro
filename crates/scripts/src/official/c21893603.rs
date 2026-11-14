// Lee the World Chalice Fairy
use crate::prelude::*;
use crate::testing::{Check, DECK, GRAVE, HAND, MZONE};

const ID: u32 = 21893603;

pub const LISTED_SERIES: &[u16] = &[SET_WORLD_CHALICE];
// Normal Summoned from the hand (a second copy in the Deck to search, a monster in the hand as cost), and in the Graveyard to return.
pub const SUPPORT: &[(u32, u16)] = &[(ID, HAND), (ID, HAND), (ID, DECK), (ID, GRAVE), (ID, MZONE), (32295838, HAND)];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_TOHAND + CATEGORY_SEARCH);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e1.set_code(duel, EVENT_SUMMON_SUCCESS);
    e1.set_property(duel, EFFECT_FLAG_DELAY);
    e1.set_count_limit(duel, 1, ID);
    e1.set_target(duel, thtg1);
    e1.set_operation(duel, thop1);
    c.register_effect(duel, e1);
    let e2 = e1.clone_effect(duel);
    e2.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 1));
    e3.set_category(duel, CATEGORY_TOHAND);
    e3.set_type(duel, EFFECT_TYPE_IGNITION);
    e3.set_range(duel, LOCATION_GRAVE);
    e3.set_count_limit(duel, 1, (ID, 1));
    e3.set_cost(duel, thcost2);
    e3.set_target(duel, thtg2);
    e3.set_operation(duel, stock::self_to_hand);
    c.register_effect(duel, e3);
}

fn thfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_WORLD_CHALICE]) && c.is_monster(duel) && c.is_able_to_hand(duel)
}

fn thtg1(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    stock::exists_target(duel, a, chk, thfilter, LOCATION_DECK, CATEGORY_TOHAND)
}

fn thop1(duel: &mut Duel, a: &Args) -> Script<()> {
    stock::search(duel, a, thfilter, LOCATION_DECK)
}

fn thcfilter(c: CardId, duel: &Duel) -> bool {
    c.is_monster(duel) && c.is_able_to_grave_as_cost(duel)
}

fn thcost2(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    stock::send_to_grave_cost(duel, a, chk, thcfilter, LOCATION_HAND | LOCATION_MZONE)
}

fn thtg2(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let c = a.e.get_handler(duel);
    if chk == 0 {
        return Ok(c.is_able_to_hand(duel));
    }
    duel.set_operation_info(0, CATEGORY_TOHAND, Some(c.into()), 1, 0, 0);
    Ok(false)
}
