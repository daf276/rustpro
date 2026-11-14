// Live☆Twin Ki-sikil Frost
use crate::prelude::*;
use crate::testing::{Check, GRAVE, HAND, MZONE, OPPONENT};

const ID: u32 = 54257392;

pub const LISTED_SERIES: &[u16] = &[SET_LIL_LA, SET_EVIL_TWIN];
// In the hand and Graveyard beside Evil★Twin Lil-la (a Lil-la monster for the hand Special Summon, an Evil★Twin monster for the draw),
// with the opponent's Elfnote Lucina to search from the Deck.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, GRAVE),
    (36609518, MZONE),
    (36609518, MZONE),
    (13597785, HAND | OPPONENT),
];
pub const CHECKS: &[Check] = &[Check::SpecialSummoned, Check::ResolvesEffect(0)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_code(duel, EFFECT_SPSUMMON_PROC);
    e1.set_range(duel, LOCATION_HAND);
    e1.set_property(duel, EFFECT_FLAG_UNCOPYABLE);
    e1.set_count_limit_full(duel, 1, ID, EFFECT_COUNT_CODE_OATH);
    e1.set_condition(duel, spcon);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_category(duel, CATEGORY_DRAW);
    e2.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e2.set_property(duel, EFFECT_FLAG_DELAY + EFFECT_FLAG_PLAYER_TARGET);
    e2.set_code(duel, EVENT_TO_HAND);
    e2.set_range(duel, LOCATION_GRAVE);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_condition(duel, drcon);
    e2.set_cost(duel, Cost::self_banish);
    e2.set_target(duel, stock::draw_target::<1>);
    e2.set_operation(duel, stock::draw);
    c.register_effect(duel, e2);
}

fn spcon(duel: &Duel, a: &Args) -> bool {
    let Some(c) = a.c else { return true };
    duel.get_location_count(c.get_controler(duel), LOCATION_MZONE) > 0
        && duel.is_existing_matching_card(
            aux::faceup_filter(|c, duel| c.is_set_card(duel, &[SET_LIL_LA])),
            a.e.get_handler_player(duel),
            LOCATION_MZONE,
            0,
            1,
            None,
        )
}

fn cfilter(c: CardId, duel: &Duel, tp: u8) -> bool {
    c.is_controler(duel, tp) && c.is_previous_location(duel, LOCATION_DECK) && c.is_reason(duel, REASON_EFFECT)
}

fn drcon(duel: &Duel, a: &Args) -> bool {
    let tp = a.tp;
    a.eg.unwrap_or_default()
        .is_exists(duel, |c, duel| cfilter(c, duel, 1 - tp), 1, None)
        && duel.is_existing_matching_card(
            aux::faceup_filter(|c, duel| c.is_set_card(duel, &[SET_EVIL_TWIN])),
            tp,
            LOCATION_MZONE,
            0,
            1,
            None,
        )
}
