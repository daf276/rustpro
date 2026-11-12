// Royal Swamp Eel
use crate::prelude::*;
use crate::testing::{Check, EXTRA, MZONE};

const ID: u32 = 37953640;

// On the field as the WATER Tuner of a Synchro Summon of White Aura Porpoise with White Dolphin, a Level 2 Fish Normal Monster.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (92409659, MZONE),
    (92409659, MZONE),
    (46956301, EXTRA),
    (46956301, EXTRA),
    (46956301, EXTRA),
];
pub const CHECKS: &[Check] = &[Check::MaterialFrom(MZONE), Check::SpecialSummonedCode(46956301)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE + EFFECT_FLAG_UNCOPYABLE);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_code(duel, EFFECT_SYNCHRO_MAT_RESTRICTION);
    e1.set_value(duel, Value::Card(|duel, _, c| i32::from(c.is_race(duel, RACE_FISH))));
    c.register_effect(duel, e1);
}
