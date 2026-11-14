// Assault Blackwing - Sayo the Rain Hider
use super::c16051717::{add_type_effects, tncon};
use super::c95040215::Procedure;
use crate::prelude::*;
use crate::testing::{Check, EXTRA, MZONE, OPPONENT};
use engine::constants::MSG_ATTACK;

const ID: u32 = 17994645;

pub const LISTED_SERIES: &[u16] = &[SET_BLACKWING];
// In the Extra Deck and on the field (Synchro Summoned from Hurricane, a Level 1 Tuner, and Key Mace, Level 1), attacked by
// Elfnote Lucina.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (ID, EXTRA),
    (ID, EXTRA),
    (36442179, MZONE),
    (36442179, MZONE),
    (1929294, MZONE),
    (1929294, MZONE),
    (13597785, MZONE | OPPONENT),
];
// Only continuous effects.
pub const CHECKS: &[Check] = &[Check::Message(MSG_ATTACK)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    Synchro::add_procedure::<Procedure<1>>(duel, c);
    c.enable_revive_limit(duel);
    add_type_effects(duel, c, tncon);
    let e3 = duel.create_effect(c);
    e3.set_type(duel, EFFECT_TYPE_SINGLE);
    e3.set_property(duel, EFFECT_FLAG_SINGLE_RANGE);
    e3.set_range(duel, LOCATION_MZONE);
    e3.set_code(duel, EFFECT_INDESTRUCTABLE_COUNT);
    e3.set_count_limit(duel, 2, 0);
    e3.set_value(duel, Value::Reason(|_, _, _, r, _| r & REASON_BATTLE != 0));
    c.register_effect(duel, e3);
}
