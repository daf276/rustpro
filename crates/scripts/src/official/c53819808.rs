// Temple of the Six
use engine::constants::MSG_ADD_COUNTER;

use crate::prelude::*;
use crate::testing::{Check, HAND};

pub const COUNTER_PLACE_LIST: &[u16] = &[COUNTER_BUSHIDO];

// Activated from the hand, with Six Samurai (Kageki, Kizan) in the hand to Summon.
pub const SUPPORT: &[(u32, u16)] = &[(2511717, HAND), (2511717, HAND), (49721904, HAND), (49721904, HAND)];
pub const CHECKS: &[Check] = &[Check::Message(MSG_ADD_COUNTER)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_counter_permit(duel, COUNTER_BUSHIDO);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_CONTINUOUS);
    e2.set_range(duel, LOCATION_FZONE);
    e2.set_code(duel, EVENT_SUMMON_SUCCESS);
    e2.set_operation(duel, ctop);
    c.register_effect(duel, e2);
    let e3 = e2.clone_effect(duel);
    e3.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    c.register_effect(duel, e3);
    let e4 = duel.create_effect(c);
    e4.set_type(duel, EFFECT_TYPE_FIELD);
    e4.set_code(duel, EFFECT_UPDATE_ATTACK);
    e4.set_range(duel, LOCATION_FZONE);
    e4.set_target_range(duel, 0, LOCATION_MZONE);
    e4.set_value(
        duel,
        Value::Card(|duel, e, _| i32::from(e.get_handler(duel).get_counter(duel, COUNTER_BUSHIDO)) * -100),
    );
    c.register_effect(duel, e4);
}

fn ctfilter(c: CardId, duel: &Duel) -> bool {
    c.is_faceup(duel) && c.is_set_card(duel, &[SET_SIX_SAMURAI])
}

fn ctop(duel: &mut Duel, a: &Args) -> Script<()> {
    if a.eg.is_some_and(|eg| eg.is_exists(duel, ctfilter, 1, None)) {
        a.e.get_handler(duel).add_counter(duel, COUNTER_BUSHIDO, 1);
    }
    Ok(())
}
