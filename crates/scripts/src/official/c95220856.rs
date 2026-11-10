// Vengeful Bog Spirit
use crate::prelude::*;
use crate::testing::HAND;

const ID: u32 = 95220856;

pub const SUPPORT: &[(u32, u16)] = &[(ID, HAND), (ID, HAND)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_FIELD);
    e2.set_code(duel, EFFECT_CANNOT_ATTACK);
    e2.set_range(duel, LOCATION_SZONE);
    e2.set_target_range(duel, LOCATION_MZONE, LOCATION_MZONE);
    e2.set_target_card(duel, |duel, _, c| {
        c.is_status(duel, STATUS_SUMMON_TURN | STATUS_FLIP_SUMMON_TURN | STATUS_SPSUMMON_TURN)
    });
    c.register_effect(duel, e2);
}
