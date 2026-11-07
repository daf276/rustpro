// Mono Synchron
use crate::prelude::*;
use crate::testing::{Check, EXTRA, MZONE};

// On the field with a Level 4 Warrior, Kojikocy, for Formula Synchron in the Extra Deck.
pub const SUPPORT: &[(u32, u16)] = &[(50091196, EXTRA), (56897896, MZONE), (1184620, MZONE)];
// EFFECT_SYNCHRO_MATERIAL_CUSTOM and EFFECT_SYNCHRO_MAT_RESTRICTION.
pub const CHECKS: &[Check] = &[Check::MaterialFrom(MZONE)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE + EFFECT_FLAG_UNCOPYABLE);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_code(duel, EFFECT_SYNCHRO_MATERIAL_CUSTOM);
    e1.set_synchro_material_operation(duel, |duel, e, _, _, sg, lv, _, _| {
        (sg.len() as u32 + e.get_handler(duel).get_level(duel) == lv + 1, true)
    });
    c.register_effect(duel, e1);
    let e4 = duel.create_effect(c);
    e4.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE + EFFECT_FLAG_UNCOPYABLE);
    e4.set_type(duel, EFFECT_TYPE_SINGLE);
    e4.set_code(duel, EFFECT_SYNCHRO_MAT_RESTRICTION);
    e4.set_value(
        duel,
        Value::Card(|duel, _, c| (c.is_level_below(duel, 4) && c.is_race(duel, RACE_WARRIOR | RACE_MACHINE)).into()),
    );
    c.register_effect(duel, e4);
}
