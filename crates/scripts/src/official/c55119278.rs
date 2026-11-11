// Morphtronic Radion
use engine::constants::MSG_ATTACK;

use crate::prelude::*;
use crate::testing::{Check, MZONE, OPPONENT};

const ID: u32 = 55119278;

// Only continuous effects, so the check is the attacks, with the opponent's attackers.
pub const SUPPORT: &[(u32, u16)] = &[(ID, MZONE), (ID, MZONE), (13597785, MZONE | OPPONENT)];
pub const CHECKS: &[Check] = &[Check::Message(MSG_ATTACK)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_target_range(duel, LOCATION_MZONE, 0);
    e1.set_code(duel, EFFECT_UPDATE_ATTACK);
    e1.set_condition(duel, |duel, a| a.e.get_handler(duel).is_attack_pos(duel));
    e1.set_target_card(duel, |duel, _, c| c.is_set_card(duel, &[SET_MORPHTRONIC]));
    e1.set_value(duel, 800);
    c.register_effect(duel, e1);
    let e2 = e1.clone_effect(duel);
    e2.set_code(duel, EFFECT_UPDATE_DEFENSE);
    e2.set_condition(duel, |duel, a| a.e.get_handler(duel).is_defense_pos(duel));
    e2.set_value(duel, 1000);
    c.register_effect(duel, e2);
}
