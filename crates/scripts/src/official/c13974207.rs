// Denko Sekka
use crate::prelude::*;
use crate::testing::{Check, HAND, MZONE, SZONE};
use engine::constants::MSG_ATTACK;

const ID: u32 = 13974207;

// On the field with face-up and Set Spells and Traps (Imperial Iron Wall, Gravity Bind) to block.
pub const SUPPORT: &[(u32, u16)] = &[(ID, MZONE), (ID, HAND), (30459350, SZONE), (85742772, SZONE)];
// Only continuous effects.
pub const CHECKS: &[Check] = &[Check::Message(MSG_ATTACK)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_code(duel, EFFECT_SPSUMMON_CONDITION);
    e1.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE + EFFECT_FLAG_UNCOPYABLE);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_FIELD);
    e2.set_code(duel, EFFECT_CANNOT_SSET);
    e2.set_property(duel, EFFECT_FLAG_PLAYER_TARGET);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_target_range(duel, 1, 1);
    e2.set_condition(duel, effcon);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_type(duel, EFFECT_TYPE_FIELD);
    e3.set_code(duel, EFFECT_CANNOT_TRIGGER);
    e3.set_property(duel, EFFECT_FLAG_SET_AVAILABLE);
    e3.set_range(duel, LOCATION_MZONE);
    e3.set_target_range(duel, LOCATION_SZONE, LOCATION_SZONE);
    e3.set_target_card(duel, |duel, _, c| c.is_facedown(duel));
    e3.set_condition(duel, effcon);
    c.register_effect(duel, e3);
}

fn effcon(duel: &Duel, a: &Args) -> bool {
    !duel.is_existing_matching_card(CardId::is_facedown, a.e.get_handler_player(duel), LOCATION_SZONE, 0, 1, None)
}
