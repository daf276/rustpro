// Frost Tiger
use crate::prelude::*;
use crate::testing::{DECK, HAND};

const ID: u32 = 51083544;
const TWIN_PILLARS_OF_ICE: u32 = 88477149;

pub const LISTED_NAMES: &[u32] = &[TWIN_PILLARS_OF_ICE];

// Copies in the hand to Normal Summon, with Neo Aqua Madoor (Level 6 WATER Spellcaster) and Twin Pillars of Ice in the Deck.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, HAND),
    (ID, HAND),
    (ID, HAND),
    (49563947, DECK),
    (49563947, DECK),
    (TWIN_PILLARS_OF_ICE, DECK),
];

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
    e1b.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    c.register_effect(duel, e1b);
}

fn thfilter(c: CardId, duel: &Duel) -> bool {
    ((c.is_level_above(duel, 5) && c.is_attribute(duel, ATTRIBUTE_WATER) && c.is_race(duel, RACE_SPELLCASTER))
        || c.is_code(duel, TWIN_PILLARS_OF_ICE))
        && c.is_able_to_hand(duel)
}

fn thtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    stock::exists_target(duel, a, chk, thfilter, LOCATION_DECK, CATEGORY_TOHAND)
}

fn thop(duel: &mut Duel, a: &Args) -> Script<()> {
    stock::search(duel, a, thfilter, LOCATION_DECK)
}
