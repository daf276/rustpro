// Ars Magna - "Citrinitas"
use crate::prelude::*;
use crate::testing::{Check, DECK, HAND, MZONE};

const ID: u32 = 37279096;

pub const LISTED_SERIES: &[u16] = &[SET_ARTMAGE, SET_DOOMZ, SET_ELFNOTE, SET_ARS_MAGNA, SET_POWER_PATRON, SET_MEDIUS];
// In hand to activate, with Purification Power Patron on the field and Ars Magna and Medius monsters in the Deck to add; the Quick
// Effects it allows are checked from the Ars Magna monsters.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (31822037, MZONE),
    (36270527, DECK),
    (62368221, DECK),
    (CARD_MEDIUS_THE_PURE, DECK),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(0), Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e0 = duel.create_effect(c);
    e0.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e0.set_code(duel, EVENT_FREE_CHAIN);
    c.register_effect(duel, e0);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_property(duel, EFFECT_FLAG_PLAYER_TARGET);
    e1.set_code(duel, EFFECT_ARS_MAGNA_CITRINITAS);
    e1.set_range(duel, LOCATION_SZONE);
    e1.set_condition(duel, effcon);
    e1.set_target_range(duel, 1, 0);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 0));
    e2.set_category(duel, CATEGORY_TOHAND + CATEGORY_SEARCH);
    e2.set_type(duel, EFFECT_TYPE_IGNITION);
    e2.set_property(duel, EFFECT_FLAG_NO_TURN_RESET);
    e2.set_range(duel, LOCATION_SZONE);
    e2.set_count_limit(duel, 1, ID);
    e2.set_target(duel, thtg);
    e2.set_operation(duel, thop);
    c.register_effect(duel, e2);
}

fn powerpatronfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_POWER_PATRON]) && c.is_link_monster(duel) && c.is_faceup(duel)
}

fn effcon(duel: &Duel, a: &Args) -> bool {
    duel.is_existing_matching_card(powerpatronfilter, a.e.get_handler_player(duel), LOCATION_MZONE, 0, 1, None)
}

fn thfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_MEDIUS, SET_ARS_MAGNA]) && c.is_monster(duel) && c.is_able_to_hand(duel)
}

fn thtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    stock::exists_target(duel, a, chk, thfilter, LOCATION_DECK, CATEGORY_TOHAND)
}

fn thop(duel: &mut Duel, a: &Args) -> Script<()> {
    stock::search(duel, a, thfilter, LOCATION_DECK)
}
