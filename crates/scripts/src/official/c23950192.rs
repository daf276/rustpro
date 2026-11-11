// Cryomancer of the Ice Barrier
use engine::constants::MSG_ATTACK;

use crate::prelude::*;
use crate::testing::{Check, MZONE, OPPONENT};

const ID: u32 = 23950192;

// On the field with Mirror Mage of the Ice Barrier, and Level 4 or higher attackers on the opponent's side.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (9396662, MZONE),
    (26202165, MZONE | OPPONENT),
    (6631034, MZONE | OPPONENT),
];
// It has no effect to resolve.
pub const CHECKS: &[Check] = &[Check::Message(MSG_ATTACK)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_FIELD);
    e2.set_code(duel, EFFECT_CANNOT_ATTACK_ANNOUNCE);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_target_range(duel, LOCATION_MZONE, LOCATION_MZONE);
    e2.set_target_card(duel, |duel, _, c| c.get_level(duel) >= 4);
    e2.set_condition(duel, con);
    c.register_effect(duel, e2);
}

fn con(duel: &Duel, a: &Args) -> bool {
    let c = a.e.get_handler(duel);
    duel.is_existing_matching_card(
        aux::faceup_filter(|c, duel| c.is_set_card(duel, &[SET_ICE_BARRIER])),
        c.get_controler(duel),
        LOCATION_MZONE,
        0,
        1,
        Some(c.into()),
    )
}
