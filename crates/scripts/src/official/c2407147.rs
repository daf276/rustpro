// Laval Volcano Handmaiden
use crate::prelude::*;
use crate::testing::{Check, DECK, GRAVE, HAND, MZONE, OPPONENT};

const ID: u32 = 2407147;

// On the field, with a Laval monster in the Graveyard, more in the Deck, and the opponent's Dark Hole to destroy it.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (45439263, GRAVE),
    (8041569, DECK),
    (8041569, DECK),
    (15169262, DECK),
    (53129443, HAND | OPPONENT),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_TOGRAVE);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e1.set_property(duel, EFFECT_FLAG_DAMAGE_STEP);
    e1.set_code(duel, EVENT_TO_GRAVE);
    e1.set_condition(duel, condition);
    e1.set_target(duel, target);
    e1.set_operation(duel, operation);
    c.register_effect(duel, e1);
}

fn cfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_LAVAL]) && c.get_code(duel) != ID
}

fn condition(duel: &Duel, a: &Args) -> bool {
    duel.is_existing_matching_card(cfilter, a.tp, LOCATION_GRAVE, 0, 1, None)
}

fn filter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_LAVAL]) && c.is_monster(duel) && c.is_able_to_grave(duel)
}

fn target(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    stock::exists_target(duel, a, chk, filter, LOCATION_DECK, CATEGORY_TOGRAVE)
}

fn operation(duel: &mut Duel, a: &Args) -> Script<()> {
    stock::send_to_grave(duel, a, filter, LOCATION_DECK)
}
