// Umi
use crate::prelude::*;
use crate::testing::{HAND, MZONE};

const ID: u32 = 22702055;

// An Aqua Normal Monster, Psychic Kappa, on the field for it to raise.
pub const SUPPORT: &[(u32, u16)] = &[(ID, HAND), (7892180, MZONE)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_FIELD);
    e2.set_code(duel, EFFECT_UPDATE_ATTACK);
    e2.set_range(duel, LOCATION_FZONE);
    e2.set_target_range(duel, LOCATION_MZONE, LOCATION_MZONE);
    e2.set_value(duel, Value::Card(val));
    c.register_effect(duel, e2);
    let e3 = e2.clone_effect(duel);
    e3.set_code(duel, EFFECT_UPDATE_DEFENSE);
    c.register_effect(duel, e3);
}

fn val(duel: &Duel, _: EffectId, c: CardId) -> i32 {
    let race = c.get_race(duel);
    if race & (RACE_FISH | RACE_SEASERPENT | RACE_THUNDER | RACE_AQUA) > 0 {
        200
    } else if race & (RACE_MACHINE | RACE_PYRO) > 0 {
        -200
    } else {
        0
    }
}
