// Dragon Horn Hunter
use crate::prelude::*;
use crate::testing::{Check, MZONE, OPPONENT, PZONE};
use engine::constants::MSG_ATTACK;

const ID: u32 = 21970285;

// In the Pendulum Zone, with Battle Ox (a Normal Monster) on both fields to attack.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, PZONE),
    (5053103, MZONE),
    (5053103, MZONE),
    (5053103, MZONE | OPPONENT),
    (5053103, MZONE | OPPONENT),
];
// Both effects are continuous and form no chain link.
pub const CHECKS: &[Check] = &[Check::Message(MSG_ATTACK)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    Pendulum::add_procedure(duel, c);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_FIELD);
    e2.set_code(duel, EFFECT_UPDATE_ATTACK);
    e2.set_range(duel, LOCATION_PZONE);
    e2.set_target_range(duel, LOCATION_MZONE, LOCATION_MZONE);
    e2.set_target_card(duel, efilter);
    e2.set_value(duel, 200);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_type(duel, EFFECT_TYPE_FIELD);
    e3.set_code(duel, EFFECT_AVOID_BATTLE_DAMAGE);
    e3.set_property(duel, EFFECT_FLAG_IGNORE_IMMUNE);
    e3.set_range(duel, LOCATION_PZONE);
    e3.set_target_range(duel, LOCATION_MZONE, 0);
    e3.set_target_card(duel, efilter);
    e3.set_value(duel, 1);
    c.register_effect(duel, e3);
}

fn efilter(duel: &Duel, _: EffectId, c: CardId) -> bool {
    c.is_type(duel, TYPE_NORMAL)
}
