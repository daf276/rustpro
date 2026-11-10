// Gren Maju Da Eiza
use crate::prelude::*;
use crate::testing::{Check, MZONE};
use engine::constants::MSG_ATTACK;

// No activated effect.
pub const SUPPORT: &[(u32, u16)] = &[(36584821, MZONE), (36584821, MZONE)];
pub const CHECKS: &[Check] = &[Check::Message(MSG_ATTACK)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_property(duel, EFFECT_FLAG_SINGLE_RANGE);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_code(duel, EFFECT_SET_ATTACK);
    e1.set_value(duel, Value::Card(value));
    c.register_effect(duel, e1);
    let e2 = e1.clone_effect(duel);
    e2.set_code(duel, EFFECT_SET_DEFENSE);
    c.register_effect(duel, e2);
}

fn value(duel: &Duel, _: EffectId, c: CardId) -> i32 {
    duel.get_field_group_count(c.get_controler(duel), LOCATION_REMOVED, 0) as i32 * 400
}
