// Flamvell Magician
use crate::prelude::*;
use crate::testing::{Check, MZONE, OPPONENT};
use engine::constants::MSG_ATTACK;

const ID: u32 = 95621257;

pub const LISTED_SERIES: &[u16] = &[SET_ALLY_OF_JUSTICE];
// Continuous effect only; on the field with Ally of Justice Clausolas (a Level 6 Normal Monster) and Sangan of the opponent's to attack.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (ID, MZONE),
    (89386122, MZONE),
    (89386122, MZONE),
    (26202165, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[Check::Message(MSG_ATTACK)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_property(duel, EFFECT_FLAG_SINGLE_RANGE);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_code(duel, EFFECT_UPDATE_ATTACK);
    e1.set_condition(duel, atkcon);
    e1.set_value(duel, 400);
    c.register_effect(duel, e1);
}

fn atkcon(duel: &Duel, a: &Args) -> bool {
    let tp = a.e.get_handler(duel).get_controler(duel);
    duel.is_existing_matching_card(
        aux::faceup_filter(|c, duel| c.is_set_card(duel, &[SET_ALLY_OF_JUSTICE])),
        tp,
        LOCATION_MZONE,
        0,
        1,
        None,
    )
}
