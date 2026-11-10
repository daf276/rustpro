// Dimensional Fissure
use crate::prelude::*;

const ID: u32 = 81674782;

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_FIELD);
    e2.set_property(
        duel,
        EFFECT_FLAG_SET_AVAILABLE + EFFECT_FLAG_IGNORE_RANGE + EFFECT_FLAG_IGNORE_IMMUNE,
    );
    e2.set_code(duel, EFFECT_TO_GRAVE_REDIRECT);
    e2.set_range(duel, LOCATION_SZONE);
    e2.set_target_card(duel, |duel, _, c| !c.is_location(duel, LOCATION_OVERLAY) && !c.is_spell_trap(duel));
    e2.set_target_range(duel, 0xff, 0xff);
    e2.set_value(duel, LOCATION_REMOVED as i32);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_type(duel, EFFECT_TYPE_FIELD);
    e3.set_code(duel, ID);
    e3.set_range(duel, LOCATION_SZONE);
    e3.set_property(duel, EFFECT_FLAG_SET_AVAILABLE);
    e3.set_target_range(duel, 0xff, 0xff);
    e3.set_target_card(duel, |duel, _, c| !c.is_public(duel));
    c.register_effect(duel, e3);
}
