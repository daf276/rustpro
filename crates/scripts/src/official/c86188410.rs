// Elemental HERO Wildheart
use engine::constants::MSG_ATTACK;

use crate::prelude::*;
use crate::testing::{Check, MZONE, OPPONENT, SZONE};

// No activated effect: it attacks into the opponent's Mirror Force and Dominus Impulse, which it is immune to.
pub const SUPPORT: &[(u32, u16)] = &[(86188410, MZONE), (44095762, SZONE | OPPONENT), (40366667, SZONE | OPPONENT)];
pub const CHECKS: &[Check] = &[Check::Message(MSG_ATTACK)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_property(duel, EFFECT_FLAG_SINGLE_RANGE);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_code(duel, EFFECT_IMMUNE_EFFECT);
    e1.set_value(duel, Value::Effect(efilter));
    c.register_effect(duel, e1);
}

fn efilter(duel: &Duel, _: EffectId, te: EffectId) -> bool {
    te.is_trap_effect(duel)
}
