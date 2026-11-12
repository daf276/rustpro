// Fairyant the Circular Sorcerer
use crate::prelude::*;
use crate::testing::{Check, EXTRA, GRAVE, HAND, MZONE};

const ID: u32 = 53416326;

// In the hand with a Plant in the Graveyard (Dark Plant), and on the field as the non-Tuner of a Level 4 Plant Synchro Summon
// (Predaplant Bal Teah) with Lunar Rabbit Omajinai.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, MZONE),
    (ID, MZONE),
    (47643326, MZONE),
    (47643326, MZONE),
    (66787942, EXTRA),
    (66787942, EXTRA),
    (13193642, GRAVE),
];
pub const CHECKS: &[Check] = &[Check::SpecialSummoned, Check::MaterialFrom(MZONE)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_property(duel, EFFECT_FLAG_UNCOPYABLE);
    e1.set_code(duel, EFFECT_SPSUMMON_PROC);
    e1.set_range(duel, LOCATION_HAND);
    e1.set_count_limit_full(duel, 1, ID, EFFECT_COUNT_CODE_OATH);
    e1.set_condition(duel, spcon);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_SINGLE);
    e2.set_property(duel, EFFECT_FLAG_SINGLE_RANGE);
    e2.set_code(duel, EFFECT_NONTUNER);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_value(
        duel,
        Value::Card(|duel, _, sc| i32::from(sc.is_race(duel, RACE_PLANT | RACE_INSECT))),
    );
    c.register_effect(duel, e2);
}

fn spcon(duel: &Duel, a: &Args) -> bool {
    let Some(_) = a.c else { return true };
    duel.is_existing_matching_card(
        |c, duel| c.is_race(duel, RACE_PLANT | RACE_INSECT),
        0,
        LOCATION_GRAVE,
        LOCATION_GRAVE,
        1,
        None,
    ) && duel.get_location_count(a.e.get_handler_player(duel), LOCATION_MZONE) > 0
}
