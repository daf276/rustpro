// Jurrac Brachis
use engine::constants::MSG_ATTACK;

use crate::prelude::*;
use crate::testing::{Check, MZONE, OPPONENT};

const ID: u32 = 8594079;

// Only a continuous effect, so the check is the attacks, with another Jurrac to enable it and the opponent's attackers.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (93170499, MZONE),
    (13597785, MZONE | OPPONENT),
    (13597785, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[Check::Message(MSG_ATTACK)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_SINGLE);
    e2.set_code(duel, EFFECT_INDESTRUCTABLE_BATTLE);
    e2.set_condition(duel, indcon);
    e2.set_value(duel, 1);
    c.register_effect(duel, e2);
}

fn indcon(duel: &Duel, a: &Args) -> bool {
    let filter = aux::faceup_filter(|c, duel| c.is_set_card(duel, &[SET_JURRAC]));
    duel.is_existing_matching_card(filter, 0, LOCATION_MZONE, LOCATION_MZONE, 1, Some(a.e.get_handler(duel).into()))
}
