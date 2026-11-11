// Clock Resonator
use crate::prelude::*;
use crate::testing::{Check, MZONE, OPPONENT};
use engine::constants::MSG_ATTACK;

const ID: u32 = 77087109;

// Continuous effect only; on the field with a monster of the opponent's (Sangan) to attack and be attacked.
pub const SUPPORT: &[(u32, u16)] = &[(ID, MZONE), (ID, MZONE), (26202165, MZONE | OPPONENT)];
pub const CHECKS: &[Check] = &[Check::Message(MSG_ATTACK)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_property(duel, EFFECT_FLAG_SINGLE_RANGE);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_code(duel, EFFECT_INDESTRUCTABLE_COUNT);
    e1.set_count_limit(duel, 1, 0);
    e1.set_value(
        duel,
        Value::Reason(|duel, e, _, r, _| r & (REASON_EFFECT + REASON_BATTLE) != 0 && e.get_handler(duel).is_defense_pos(duel)),
    );
    c.register_effect(duel, e1);
}
