// Defender of the Ice Barrier
use engine::constants::MSG_ATTACK;

use crate::prelude::*;
use crate::testing::{Check, MZONE, OPPONENT};

const ID: u32 = 82498947;

pub const LISTED_SERIES: &[u16] = &[SET_ICE_BARRIER];
// On the field with Mirror Mage of the Ice Barrier, and a monster on the opponent's side that cannot attack into them.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (9396662, MZONE),
    (26202165, MZONE | OPPONENT),
    (6631034, MZONE | OPPONENT),
];
// It has no effect to resolve.
pub const CHECKS: &[Check] = &[Check::Message(MSG_ATTACK)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_code(duel, EFFECT_CANNOT_ATTACK_ANNOUNCE);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_target_range(duel, 0, LOCATION_MZONE);
    e1.set_target_card(duel, |duel, e, c| c.get_attack(duel) >= e.get_handler(duel).get_defense(duel));
    e1.set_condition(duel, con);
    c.register_effect(duel, e1);
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
