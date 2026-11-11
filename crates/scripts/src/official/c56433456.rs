// The Sanctuary in the Sky
use crate::prelude::*;

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_FIELD);
    e2.set_code(duel, EFFECT_AVOID_BATTLE_DAMAGE);
    e2.set_property(duel, EFFECT_FLAG_IGNORE_IMMUNE);
    e2.set_range(duel, LOCATION_FZONE);
    e2.set_target_range(duel, LOCATION_MZONE, LOCATION_MZONE);
    e2.set_target_card(duel, |duel, _, c| c.is_race(duel, RACE_FAIRY));
    e2.set_value(duel, 1);
    c.register_effect(duel, e2);
}
