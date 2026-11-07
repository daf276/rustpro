// Chaos Necromancer
use engine::constants::MSG_ATTACK;

use crate::prelude::*;
use crate::testing::{Check, GRAVE, MZONE, OPPONENT};

const ID: u32 = 1434352;

// No activated effect: it attacks a weak monster of the opponent's, with Sangan in the Graveyard for the ATK.
pub const SUPPORT: &[(u32, u16)] = &[(ID, MZONE), (26202165, GRAVE), (26202165, GRAVE), (26202165, MZONE | OPPONENT)];
pub const CHECKS: &[Check] = &[Check::Message(MSG_ATTACK)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_code(duel, EFFECT_SET_ATTACK);
    e1.set_property(duel, EFFECT_FLAG_SINGLE_RANGE);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_value(duel, Value::Card(atkval));
    c.register_effect(duel, e1);
}

fn atkval(duel: &Duel, _: EffectId, c: CardId) -> i32 {
    duel.get_matching_group_count(CardId::is_monster, c.get_controler(duel), LOCATION_GRAVE, 0, None) as i32 * 300
}
