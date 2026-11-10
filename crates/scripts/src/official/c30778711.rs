// Shadow Ghoul
use engine::constants::MSG_ATTACK;

use crate::prelude::*;
use crate::testing::{Check, GRAVE, MZONE, OPPONENT};

const ID: u32 = 30778711;

// Two on the field and one in the Graveyard, and the opponent's Sangan to attack.
pub const SUPPORT: &[(u32, u16)] = &[(ID, MZONE), (ID, MZONE), (ID, GRAVE), (26202165, MZONE | OPPONENT)];
// It has no effect to resolve.
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
    duel.get_matching_group_count(CardId::is_monster, c.get_controler(duel), LOCATION_GRAVE, 0, None) as i32 * 100
}
