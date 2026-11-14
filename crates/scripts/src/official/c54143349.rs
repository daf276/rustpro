// Radiant Typhoon Eldam
use crate::prelude::*;
use crate::testing::{Check, DECK, GRAVE, HAND};

const ID: u32 = 54143349;

pub const LISTED_SERIES: &[u16] = &[SET_RADIANT_TYPHOON];
pub const LISTED_NAMES: &[u32] = &[CARD_MYSTICAL_SPACE_TYPHOON, ID];

// A copy in the hand, and Mystical Space Typhoon in the Deck to add and in the GY for the Special Summon.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (CARD_MYSTICAL_SPACE_TYPHOON, DECK),
    (CARD_MYSTICAL_SPACE_TYPHOON, GRAVE),
];
pub const CHECKS: &[Check] = &[Check::SpecialSummoned, Check::ResolvesEffect(aux::stringid(ID, 1))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_property(duel, EFFECT_FLAG_UNCOPYABLE);
    e1.set_code(duel, EFFECT_SPSUMMON_PROC);
    e1.set_range(duel, LOCATION_HAND);
    e1.set_count_limit_full(duel, 1, ID, EFFECT_COUNT_CODE_OATH);
    e1.set_condition(duel, selfspcon);
    c.register_effect(duel, e1);
    let e2a = duel.create_effect(c);
    e2a.set_description(duel, aux::stringid(ID, 1));
    e2a.set_category(duel, CATEGORY_TOHAND + CATEGORY_SEARCH);
    e2a.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e2a.set_property(duel, EFFECT_FLAG_DELAY);
    e2a.set_code(duel, EVENT_SUMMON_SUCCESS);
    e2a.set_count_limit(duel, 1, (ID, 1));
    e2a.set_target(duel, thtg);
    e2a.set_operation(duel, thop);
    c.register_effect(duel, e2a);
    let e2b = e2a.clone_effect(duel);
    e2b.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    c.register_effect(duel, e2b);
}

pub fn selfspcon(duel: &Duel, a: &Args) -> bool {
    let Some(c) = a.c else { return true };
    let tp = c.get_controler(duel);
    duel.get_location_count(tp, LOCATION_MZONE) > 0
        && (duel.is_existing_matching_card(
            |c, duel| c.is_code(duel, CARD_MYSTICAL_SPACE_TYPHOON),
            tp,
            LOCATION_GRAVE,
            0,
            1,
            None,
        ) || !duel.is_existing_matching_card(CardId::is_spell_trap, tp, 0, LOCATION_ONFIELD, 1, None))
}

fn thfilter(c: CardId, duel: &Duel) -> bool {
    ((c.is_set_card(duel, &[SET_RADIANT_TYPHOON]) && c.is_monster(duel)) || c.is_code(duel, CARD_MYSTICAL_SPACE_TYPHOON))
        && c.is_able_to_hand(duel)
        && !c.is_code(duel, ID)
}

fn thtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    stock::exists_target(duel, a, chk, thfilter, LOCATION_DECK, CATEGORY_TOHAND)
}

fn thop(duel: &mut Duel, a: &Args) -> Script<()> {
    stock::search(duel, a, thfilter, LOCATION_DECK)
}
