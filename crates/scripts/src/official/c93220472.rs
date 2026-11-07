// Cave Dragon
use crate::prelude::*;
use crate::testing::{Check, HAND, MZONE};
use engine::constants::MSG_ATTACK;

const ID: u32 = 93220472;

// On the field with a copy as the Dragon it needs to attack, and one in the hand for the Normal Summon limit.
pub const SUPPORT: &[(u32, u16)] = &[(ID, MZONE), (ID, MZONE), (ID, HAND)];
// It has no effect to resolve.
pub const CHECKS: &[Check] = &[Check::Message(MSG_ATTACK)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_code(duel, EFFECT_CANNOT_SUMMON);
    e1.set_condition(duel, |duel, a| {
        duel.get_field_group_count(a.e.get_handler_player(duel), LOCATION_MZONE, 0) > 0
    });
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_SINGLE);
    e2.set_code(duel, EFFECT_CANNOT_ATTACK);
    e2.set_condition(duel, atkcon);
    c.register_effect(duel, e2);
}

fn atkcon(duel: &Duel, a: &Args) -> bool {
    let c = a.e.get_handler(duel);
    !duel.is_existing_matching_card(
        aux::faceup_filter(|c, duel| c.is_race(duel, RACE_DRAGON)),
        a.e.get_handler_player(duel),
        LOCATION_MZONE,
        0,
        1,
        Some(c.into()),
    )
}
