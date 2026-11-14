// Snake-Eye Birch
use crate::prelude::*;
use crate::testing::{Check, DECK, HAND, MZONE};

use super::c9674034::{spcost, spop, sptg};

const ID: u32 = 12058741;

pub const LISTED_NAMES: &[u32] = &[ID];
pub const LISTED_SERIES: &[u16] = &[SET_SNAKE_EYE];

const SNAKE_EYE_ASH: u32 = 9674034;
const SNAKE_EYES_POPLAR: u32 = 90241276;

// In the hand and on the field with Snake-Eye Ash (FIRE) and Kitchen Dragonmaid to send, and Snake-Eye monsters in the Deck.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, MZONE),
    (SNAKE_EYE_ASH, MZONE),
    (SNAKE_EYE_ASH, DECK),
    (SNAKE_EYES_POPLAR, DECK),
    (SNAKE_EYES_POPLAR, DECK),
    (16960120, MZONE),
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
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e2.set_type(duel, EFFECT_TYPE_QUICK_O);
    e2.set_code(duel, EVENT_FREE_CHAIN);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_hint_timing(duel, 0, TIMINGS_CHECK_MONSTER_E);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_condition(duel, |duel, a| duel.is_turn_player(1 - a.tp));
    e2.set_cost(duel, spcost);
    e2.set_target(duel, sptg::<ID>);
    e2.set_operation(duel, spop::<ID>);
    c.register_effect(duel, e2);
}

fn selfspcon(duel: &Duel, a: &Args) -> bool {
    let Some(c) = a.c else { return true };
    let tp = c.get_controler(duel);
    duel.get_location_count(tp, LOCATION_MZONE) > 0
        && duel.is_existing_matching_card(
            aux::faceup_filter(|c, duel| c.is_attribute(duel, ATTRIBUTE_FIRE)),
            tp,
            LOCATION_MZONE,
            0,
            1,
            None,
        )
}
