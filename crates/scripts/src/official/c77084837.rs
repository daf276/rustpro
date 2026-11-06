// Inaba White Rabbit
use crate::prelude::*;

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    Spirit::add_procedure(duel, c, &[EVENT_SUMMON_SUCCESS, EVENT_FLIP]);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE + EFFECT_FLAG_UNCOPYABLE);
    e1.set_code(duel, EFFECT_SPSUMMON_CONDITION);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_SINGLE);
    e2.set_code(duel, EFFECT_DIRECT_ATTACK);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_type(duel, EFFECT_TYPE_SINGLE);
    e3.set_code(duel, EFFECT_CANNOT_SELECT_BATTLE_TARGET);
    e3.set_value(duel, 1);
    c.register_effect(duel, e3);
}
