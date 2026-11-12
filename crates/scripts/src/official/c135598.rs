// Key Mouse
use crate::prelude::*;
use crate::testing::{Check, DECK, MZONE, OPPONENT};

const ID: u32 = 135598;

// Copies on the field for the opponent's attackers (Pesanta the Titan of Troubadours, Elfnote Lucina) to destroy in battle, with Beast
// Normal Monsters (Nekogal #1, Burglar) in the Deck to search.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (ID, MZONE),
    (ID, MZONE),
    (1761063, DECK),
    (1761063, DECK),
    (6297941, DECK),
    (66429798, MZONE | OPPONENT),
    (66429798, MZONE | OPPONENT),
    (13597785, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_TOHAND + CATEGORY_SEARCH);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e1.set_code(duel, EVENT_BATTLE_DESTROYED);
    e1.set_condition(duel, condition);
    e1.set_target(duel, target);
    e1.set_operation(duel, operation);
    c.register_effect(duel, e1);
}

fn condition(duel: &Duel, a: &Args) -> bool {
    let c = a.e.get_handler(duel);
    c.is_location(duel, LOCATION_GRAVE) && c.is_reason(duel, REASON_BATTLE)
}

fn filter(c: CardId, duel: &Duel) -> bool {
    c.is_level_below(duel, 3) && c.is_race(duel, RACE_BEAST) && c.is_able_to_hand(duel)
}

fn target(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    stock::exists_target(duel, a, chk, filter, LOCATION_DECK, CATEGORY_TOHAND)
}

fn operation(duel: &mut Duel, a: &Args) -> Script<()> {
    stock::search(duel, a, filter, LOCATION_DECK)
}
