// Mono Synchron
use crate::prelude::*;

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
