// Blue-Eyes Chaos MAX Dragon
use crate::prelude::*;
use crate::testing::{Check, GRAVE, HAND, MZONE, OPPONENT};
use engine::constants::MSG_ATTACK;

const ID: u32 = 55410871;

pub const LISTED_NAMES: &[u32] = &[21082832];

// In the hand, Ritual Summoned by Chaos Form with Blue-Eyes White Dragon from the Graveyard, and an opponent's monster to fight.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (21082832, HAND),
    (CARD_BLUEEYES_W_DRAGON, GRAVE),
    (26202165, MZONE | OPPONENT),
];
// It has only continuous effects.
pub const CHECKS: &[Check] = &[Check::SpecialSummoned, Check::Message(MSG_ATTACK)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE + EFFECT_FLAG_UNCOPYABLE);
    e1.set_code(duel, EFFECT_SPSUMMON_CONDITION);
    e1.set_value(duel, Value::SpSummonCondition(aux::ritlimit));
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_SINGLE);
    e2.set_property(duel, EFFECT_FLAG_SINGLE_RANGE);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_code(duel, EFFECT_CANNOT_BE_EFFECT_TARGET);
    e2.set_value(duel, Value::EffectPlayer(aux::tgoval));
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_type(duel, EFFECT_TYPE_SINGLE);
    e3.set_code(duel, EFFECT_INDESTRUCTABLE_EFFECT);
    e3.set_property(duel, EFFECT_FLAG_SINGLE_RANGE);
    e3.set_range(duel, LOCATION_MZONE);
    e3.set_value(duel, Value::EffectPlayer(indval));
    c.register_effect(duel, e3);
    let e4 = duel.create_effect(c);
    e4.set_type(duel, EFFECT_TYPE_SINGLE);
    e4.set_code(duel, EFFECT_PIERCE);
    e4.set_value(duel, DOUBLE_DAMAGE as i32);
    c.register_effect(duel, e4);
}

fn indval(duel: &Duel, e: EffectId, _: EffectId, tp: u8) -> bool {
    tp != e.get_handler_player(duel)
}
