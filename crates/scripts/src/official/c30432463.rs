// Dimonno Vaalmonica
use super::c3048768::{cpcost, cpop, cptg, ctop, pzcost, pzop, pztg};
use crate::prelude::*;
use crate::testing::{Check, DECK, EXTRA, GRAVE, HAND, MZONE, OPPONENT};
use engine::constants::MSG_ADD_COUNTER;

const ID: u32 = 30432463;

pub const COUNTER_PLACE_LIST: &[u16] = &[COUNTER_RESONANCE];
const ANGELLO: u32 = 3048768;

pub const LISTED_NAMES: &[u32] = &[ANGELLO];

// Copies in the hand to place with Angello in the Pendulum Zone (which needs both zones free), Vaalmonica Spells in the Graveyard to copy, Link Monsters in the
// Extra Deck, and the opponent's monsters to attack.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, HAND),
    (ANGELLO, DECK),
    (ANGELLO, DECK),
    (65496951, HAND),
    (42193638, HAND),
    (5605529, GRAVE),
    (78598237, GRAVE),
    (1340142, EXTRA),
    (2815176, EXTRA),
    (76821171, EXTRA),
    (26202165, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
    Check::Message(MSG_ADD_COUNTER),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    Pendulum::add_procedure(duel, c);
    c.enable_counter_permit_full(duel, COUNTER_RESONANCE, LOCATION_PZONE as u16);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_CONTINUOUS);
    e1.set_code(duel, EVENT_DAMAGE);
    e1.set_range(duel, LOCATION_PZONE);
    e1.set_condition(duel, |duel, a| {
        a.ep == a.tp
            && a.r & REASON_EFFECT > 0
            && duel.is_existing_matching_card(
                |c, duel| c.is_original_race(duel, RACE_FAIRY),
                a.tp,
                LOCATION_PZONE,
                0,
                1,
                Some(a.e.get_handler(duel).into()),
            )
    });
    e1.set_operation(duel, ctop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_FIELD);
    e2.set_code(duel, EFFECT_UPDATE_ATTACK);
    e2.set_range(duel, LOCATION_PZONE);
    e2.set_target_range(duel, 0, LOCATION_MZONE);
    e2.set_value(
        duel,
        Value::Card(|duel, e, _| duel.get_counter(e.get_handler_player(duel), LOCATION_ONFIELD as u8, 0, COUNTER_RESONANCE) as i32 * -100),
    );
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 0));
    e3.set_type(duel, EFFECT_TYPE_IGNITION);
    e3.set_range(duel, LOCATION_HAND);
    e3.set_count_limit(duel, 1, (ID, 0));
    e3.set_cost(duel, pzcost);
    e3.set_target(duel, pztg::<ANGELLO>);
    e3.set_operation(duel, pzop::<ANGELLO>);
    c.register_effect(duel, e3);
    let e4 = duel.create_effect(c);
    e4.set_description(duel, aux::stringid(ID, 1));
    e4.set_type(duel, EFFECT_TYPE_IGNITION);
    e4.set_range(duel, LOCATION_MZONE);
    e4.set_count_limit(duel, 1, (ID, 1));
    e4.set_condition(duel, |duel, a| a.e.get_handler(duel).has_flag_effect(duel, ID));
    e4.set_cost(duel, cpcost);
    e4.set_target(duel, cptg);
    e4.set_operation(duel, cpop::<2>);
    c.register_effect(duel, e4);
    aux::global_check(duel, ID, |duel| {
        let ge1 = duel.create_effect(c);
        ge1.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_CONTINUOUS);
        ge1.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE);
        ge1.set_code(duel, EVENT_SUMMON_SUCCESS);
        ge1.set_label(duel, i64::from(ID));
        ge1.set_operation(duel, aux::sumreg);
        duel.register_effect(ge1, 0);
        let ge2 = ge1.clone_effect(duel);
        ge2.set_code(duel, EVENT_SPSUMMON_SUCCESS);
        duel.register_effect(ge2, 0);
    });
}
