// Al-Lumi'raj
use crate::prelude::*;
use crate::testing::{Check, MZONE, OPPONENT};
use engine::constants::MSG_ATTACK;

const ID: u32 = 25795273;

// On the field, with attackers on both sides.
pub const SUPPORT: &[(u32, u16)] = &[(ID, MZONE), (13597785, MZONE | OPPONENT), (13597785, MZONE)];
// The ATK and DEF changes show no message of their own.
pub const CHECKS: &[Check] = &[Check::Message(MSG_ATTACK)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_code(duel, EFFECT_UPDATE_ATTACK);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_target_range(duel, LOCATION_MZONE, LOCATION_MZONE);
    e1.set_value(duel, Value::Card(val));
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_FIELD);
    e2.set_code(duel, EFFECT_UPDATE_DEFENSE);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_target_range(duel, LOCATION_MZONE, LOCATION_MZONE);
    e2.set_value(duel, Value::Card(val));
    c.register_effect(duel, e2);
}

fn val(duel: &Duel, _: EffectId, c: CardId) -> i32 {
    if c.is_type(duel, TYPE_XYZ) {
        c.get_rank(duel) as i32 * -300
    } else {
        c.get_level(duel) as i32 * -300
    }
}
