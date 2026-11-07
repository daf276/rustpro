// Buster Blader
use engine::constants::MSG_ATTACK;

use crate::prelude::*;
use crate::testing::{Check, GRAVE, MZONE, OPPONENT};

const ID: u32 = 78193831;

// No activated effect: it attacks a weak monster of the opponent's, with Dragons of the opponent's in the Graveyard for the ATK.
pub const SUPPORT: &[(u32, u16)] = &[(ID, MZONE), (89631139, GRAVE | OPPONENT), (26202165, MZONE | OPPONENT)];
pub const CHECKS: &[Check] = &[Check::Message(MSG_ATTACK)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_property(duel, EFFECT_FLAG_SINGLE_RANGE);
    e1.set_code(duel, EFFECT_UPDATE_ATTACK);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_value(duel, Value::Card(val));
    c.register_effect(duel, e1);
}

fn filter(c: CardId, duel: &Duel) -> bool {
    c.is_race(duel, RACE_DRAGON) && (c.is_location(duel, LOCATION_GRAVE) || c.is_faceup(duel))
}

fn val(duel: &Duel, _: EffectId, c: CardId) -> i32 {
    duel.get_matching_group_count(filter, c.get_controler(duel), 0, LOCATION_GRAVE | LOCATION_MZONE, None) as i32 * 500
}
