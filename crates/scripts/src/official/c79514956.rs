// Elegy the Melodious Diva
use crate::prelude::*;
use crate::testing::{Check, DECK, EXTRA, MZONE};

const ID: u32 = 79514956;
const BLOOM_HARMONIST: u32 = 34974462;

// Special Summoned by Bloom Harmonist, Link Summoned from Fairy Normal Monsters.
pub const SUPPORT: &[(u32, u16)] = &[
    (BLOOM_HARMONIST, EXTRA),
    (BLOOM_HARMONIST, EXTRA),
    (BLOOM_HARMONIST, EXTRA),
    (1929294, MZONE),
    (1929294, MZONE),
    (27288416, MZONE),
    (27288416, MZONE),
    (ID, DECK),
    (ID, DECK),
    (40502912, DECK),
    (40502912, DECK),
];
pub const CHECKS: &[Check] = &[Check::SpecialSummoned];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_code(duel, EFFECT_INDESTRUCTABLE_EFFECT);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_target_range(duel, LOCATION_MZONE, 0);
    e1.set_target_card(duel, |duel, _, c| {
        c.is_set_card(duel, &[SET_MELODIOUS]) && c.is_special_summoned(duel)
    });
    e1.set_value(duel, 1);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_FIELD);
    e2.set_code(duel, EFFECT_UPDATE_ATTACK);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_target_range(duel, LOCATION_MZONE, 0);
    e2.set_target_card(duel, |duel, _, c| c.is_race(duel, RACE_FAIRY));
    e2.set_value(duel, 300);
    e2.set_condition(duel, |duel, a| a.e.get_handler(duel).is_special_summoned(duel));
    c.register_effect(duel, e2);
}
