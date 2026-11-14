// Satellarknight Deneb
use crate::prelude::*;
use crate::testing::{Check, DECK, HAND};

const ID: u32 = 75878039;

pub const LISTED_SERIES: &[u16] = &[SET_TELLARKNIGHT];
pub const LISTED_NAMES: &[u32] = &[ID];

// Satellarknight Unukalhai in the Deck to search.
pub const SUPPORT: &[(u32, u16)] = &[(ID, HAND), (ID, HAND), (1050186, DECK), (1050186, DECK)];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1a = duel.create_effect(c);
    e1a.set_description(duel, aux::stringid(ID, 0));
    e1a.set_category(duel, CATEGORY_TOHAND + CATEGORY_SEARCH);
    e1a.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e1a.set_property(duel, EFFECT_FLAG_DELAY);
    e1a.set_code(duel, EVENT_SUMMON_SUCCESS);
    e1a.set_count_limit(duel, 1, ID);
    e1a.set_target(duel, thtg);
    e1a.set_operation(duel, thop);
    c.register_effect(duel, e1a);
    let e1b = e1a.clone_effect(duel);
    e1b.set_code(duel, EVENT_FLIP_SUMMON_SUCCESS);
    c.register_effect(duel, e1b);
    let e1c = e1a.clone_effect(duel);
    e1c.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    c.register_effect(duel, e1c);
}

fn thfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_TELLARKNIGHT]) && c.is_monster(duel) && !c.is_code(duel, ID) && c.is_able_to_hand(duel)
}

fn thtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    stock::exists_target(duel, a, chk, thfilter, LOCATION_DECK, CATEGORY_TOHAND)
}

fn thop(duel: &mut Duel, a: &Args) -> Script<()> {
    stock::search(duel, a, thfilter, LOCATION_DECK)
}
