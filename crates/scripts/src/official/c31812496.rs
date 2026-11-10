// Stone Statue of the Aztecs
use engine::constants::MSG_DAMAGE;

use crate::prelude::*;
use crate::testing::{Check, MZONE, OPPONENT};

const ID: u32 = 31812496;

// On the field against Lucina, to take battle damage when attacked.
pub const SUPPORT: &[(u32, u16)] = &[(ID, MZONE), (ID, MZONE), (13597785, MZONE | OPPONENT), (13597785, MZONE | OPPONENT)];
// It has no effect to resolve.
pub const CHECKS: &[Check] = &[Check::Message(MSG_DAMAGE)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_code(duel, EFFECT_CHANGE_BATTLE_DAMAGE);
    e1.set_condition(duel, |duel, a| duel.get_attack_target() == Some(a.e.get_handler(duel)));
    e1.set_value(duel, Value::BattleDamage(aux::change_battle_damage::<1, DOUBLE_DAMAGE>));
    c.register_effect(duel, e1);
}
