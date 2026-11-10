// Maha Vailo
use engine::constants::MSG_ATTACK;

use crate::prelude::*;
use crate::testing::{Check, HAND, MZONE};

const ID: u32 = 93013676;

// On the field with Mage Power and Axe of Despair in the hand to equip.
pub const SUPPORT: &[(u32, u16)] = &[(ID, MZONE), (83746708, HAND), (83746708, HAND), (40619825, HAND)];
pub const CHECKS: &[Check] = &[Check::Message(MSG_ATTACK)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_property(duel, EFFECT_FLAG_SINGLE_RANGE);
    e1.set_code(duel, EFFECT_UPDATE_ATTACK);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_value(duel, Value::Card(|duel, _, c| c.get_equip_count(duel) as i32 * 500));
    c.register_effect(duel, e1);
}
