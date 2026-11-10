// Drillago
use engine::constants::MSG_ATTACK;

use crate::prelude::*;
use crate::testing::{Check, MZONE, OPPONENT};

const ID: u32 = 99050989;

// No activated effect: it attacks directly past the opponent's Battle Ox (1700 ATK).
pub const SUPPORT: &[(u32, u16)] = &[(ID, MZONE), (ID, MZONE), (5053103, MZONE | OPPONENT)];
pub const CHECKS: &[Check] = &[Check::Message(MSG_ATTACK)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_code(duel, EFFECT_DIRECT_ATTACK);
    e1.set_condition(duel, dircon);
    c.register_effect(duel, e1);
}

fn filter(c: CardId, duel: &Duel) -> bool {
    c.get_attack(duel) < 1600 || c.is_facedown(duel)
}

fn dircon(duel: &Duel, a: &Args) -> bool {
    let tp = a.e.get_handler_player(duel);
    duel.get_field_group_count(tp, 0, LOCATION_SZONE) == 0 && !duel.is_existing_matching_card(filter, tp, 0, LOCATION_MZONE, 1, None)
}
