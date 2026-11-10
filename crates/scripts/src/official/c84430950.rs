// Armed Samurai - Ben Kei
use engine::constants::MSG_ATTACK;

use crate::prelude::*;
use crate::testing::{Check, HAND, MZONE, OPPONENT};

const ID: u32 = 84430950;

// No activated effect: it attacks the opponent's monsters, equipped with United We Stand and Axe of Despair for extra attacks.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (56747793, HAND),
    (40619825, HAND),
    (43989315, MZONE | OPPONENT),
    (43989315, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[Check::Message(MSG_ATTACK)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_property(duel, EFFECT_FLAG_SINGLE_RANGE);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_code(duel, EFFECT_EXTRA_ATTACK);
    e1.set_value(duel, Value::Card(val));
    c.register_effect(duel, e1);
}

fn val(duel: &Duel, _: EffectId, c: CardId) -> i32 {
    c.get_equip_count(duel) as i32
}
