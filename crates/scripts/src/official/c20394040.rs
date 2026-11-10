// Lava Battleguard
use crate::prelude::*;
use crate::testing::{Check, MZONE};
use engine::constants::MSG_ATTACK;

const ID: u32 = 20394040;
const SWAMP_BATTLEGUARD: u32 = 40453765;

pub const LISTED_NAMES: &[u32] = &[SWAMP_BATTLEGUARD];

// Continuous only: the Swamp Battleguards on the field are what boost it.
pub const SUPPORT: &[(u32, u16)] = &[(ID, MZONE), (SWAMP_BATTLEGUARD, MZONE), (SWAMP_BATTLEGUARD, MZONE)];
pub const CHECKS: &[Check] = &[Check::Message(MSG_ATTACK)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_property(duel, EFFECT_FLAG_SINGLE_RANGE);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_code(duel, EFFECT_UPDATE_ATTACK);
    e1.set_value(duel, Value::Card(value));
    c.register_effect(duel, e1);
}

fn value(duel: &Duel, _: EffectId, c: CardId) -> i32 {
    let filter = aux::faceup_filter(|c, duel| c.is_code(duel, SWAMP_BATTLEGUARD));
    500 * duel.get_matching_group_count(filter, c.get_controler(duel), LOCATION_MZONE, 0, None) as i32
}
