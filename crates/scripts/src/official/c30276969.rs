// Reese the Ice Mistress
use crate::prelude::*;
use crate::testing::Check;
use engine::constants::MSG_ATTACK;

pub const CHECKS: &[Check] = &[Check::Message(MSG_ATTACK)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_code(duel, EFFECT_INDESTRUCTABLE_BATTLE);
    e1.set_value(duel, Value::Card(indes));
    c.register_effect(duel, e1);
}

fn indes(duel: &Duel, _e: EffectId, c: CardId) -> i32 {
    c.is_level_above(duel, 4).into()
}
