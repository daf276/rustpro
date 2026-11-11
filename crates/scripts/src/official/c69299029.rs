// Treasures of the Kings
use crate::prelude::*;
use crate::testing::{Check, DECK, HAND, SZONE};

const ID: u32 = 69299029;

pub const LISTED_NAMES: &[u32] = &[CARD_TEMPLE_OF_THE_KINGS];
// In the hand to activate and Set Embodiment of Apophis from the Deck, with Set Traps for the Field Zone effect, and monsters
// that mention Temple of the Kings (The Man with the Mark, Mystical Beast of Serket) in the Deck to search.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, HAND),
    (28649820, DECK),
    (28649820, DECK),
    (28649820, SZONE),
    (28649820, SZONE),
    (97522863, DECK),
    (89194033, DECK),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_SET);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_count_limit_full(duel, 1, ID, EFFECT_COUNT_CODE_OATH);
    e1.set_operation(duel, activate);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_SINGLE);
    e2.set_property(duel, EFFECT_FLAG_SINGLE_RANGE);
    e2.set_code(duel, EFFECT_CHANGE_CODE);
    e2.set_range(duel, LOCATION_FZONE);
    e2.set_value(duel, CARD_TEMPLE_OF_THE_KINGS as i32);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 1));
    e3.set_category(duel, CATEGORY_TOHAND + CATEGORY_SEARCH);
    e3.set_type(duel, EFFECT_TYPE_IGNITION);
    e3.set_range(duel, LOCATION_FZONE);
    e3.set_count_limit(duel, 1, 0);
    e3.set_condition(duel, thcon);
    e3.set_target(duel, thtg);
    e3.set_operation(duel, thop);
    c.register_effect(duel, e3);
}

fn setfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_APOPHIS]) && c.is_trap(duel) && c.is_ssetable(duel)
}

fn activate(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    let g = duel.get_matching_group(setfilter, tp, LOCATION_DECK, 0, None);
    if !g.is_empty() && duel.select_yes_no(tp, aux::stringid(ID, 2))? {
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_SET);
        let sg = g.select(duel, tp, 1, 1, None)?;
        duel.sset(tp, sg)?;
    }
    Ok(())
}

fn thcon(duel: &Duel, a: &Args) -> bool {
    let tp = a.tp;
    duel.is_existing_matching_card(CardId::is_facedown, tp, LOCATION_ONFIELD, 0, 2, None)
        || duel.is_existing_matching_card(CardId::is_trap, tp, LOCATION_GRAVE, 0, 1, None)
}

fn thfilter(c: CardId, duel: &Duel) -> bool {
    c.is_monster(duel) && c.lists_code(duel, CARD_TEMPLE_OF_THE_KINGS) && c.is_able_to_hand(duel)
}

fn thtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    stock::exists_target(duel, a, chk, thfilter, LOCATION_DECK, CATEGORY_TOHAND)
}

fn thop(duel: &mut Duel, a: &Args) -> Script<()> {
    stock::search(duel, a, thfilter, LOCATION_DECK)
}
