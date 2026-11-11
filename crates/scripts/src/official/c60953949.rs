// Ancient Gear Box
use crate::prelude::*;
use crate::testing::{Check, DECK, EXTRA, MZONE};

const ID: u32 = 60953949;

// In the Deck, searched by Ancient Gear Ballista (Link Summoned in the Extra Deck with Mighty Guard, a Machine Normal Monster,
// on the field), with Mighty Guard in the Deck to search in turn.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, DECK),
    (ID, DECK),
    (62327910, DECK),
    (62327910, DECK),
    (62327910, DECK),
    (62327910, MZONE),
    (62327910, MZONE),
    (62327910, MZONE),
    (10547580, EXTRA),
    (10547580, EXTRA),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_TOHAND + CATEGORY_SEARCH);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e1.set_property(duel, EFFECT_FLAG_DAMAGE_STEP + EFFECT_FLAG_DELAY);
    e1.set_code(duel, EVENT_TO_HAND);
    e1.set_count_limit(duel, 1, ID);
    e1.set_condition(duel, thcon);
    e1.set_target(duel, thtg);
    e1.set_operation(duel, thop);
    c.register_effect(duel, e1);
}

fn thcon(duel: &Duel, a: &Args) -> bool {
    let c = a.e.get_handler(duel);
    !c.is_reason(duel, REASON_DRAW) && c.is_previous_location(duel, LOCATION_DECK | LOCATION_GRAVE)
}

fn filter(c: CardId, duel: &Duel) -> bool {
    (c.get_attack(duel) == 500 || c.get_defense(duel) == 500)
        && c.get_code(duel) != ID
        && c.is_race(duel, RACE_MACHINE)
        && c.is_attribute(duel, ATTRIBUTE_EARTH)
        && c.is_able_to_hand(duel)
}

fn thtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    stock::exists_target(duel, a, chk, filter, LOCATION_DECK, CATEGORY_TOHAND)
}

fn thop(duel: &mut Duel, a: &Args) -> Script<()> {
    stock::search(duel, a, filter, LOCATION_DECK)
}
