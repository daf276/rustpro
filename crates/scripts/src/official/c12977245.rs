// Altergeist Fifinellag
use crate::prelude::*;
use crate::testing::{Check, MZONE, OPPONENT};
use engine::constants::MSG_ATTACK;

const ID: u32 = 12977245;

pub const LISTED_SERIES: &[u16] = &[SET_ALTERGEIST];
pub const LISTED_NAMES: &[u32] = &[ID];

// On the field with Altergeist Malwisp to be protected, against the opponent's attackers.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (27132400, MZONE),
    (13597785, MZONE | OPPONENT),
    (26202165, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[Check::Message(MSG_ATTACK)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_target_range(duel, 0, LOCATION_MZONE);
    e1.set_code(duel, EFFECT_CANNOT_SELECT_BATTLE_TARGET);
    e1.set_value(duel, Value::Card(|duel, _, c| i32::from(imtg(c, duel))));
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_FIELD);
    e2.set_code(duel, EFFECT_CANNOT_BE_EFFECT_TARGET);
    e2.set_property(duel, EFFECT_FLAG_IGNORE_IMMUNE);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_target_range(duel, LOCATION_MZONE, 0);
    e2.set_target_card(duel, |duel, _, c| imtg(c, duel));
    e2.set_value(duel, Value::EffectPlayer(aux::tgoval));
    c.register_effect(duel, e2);
}

fn imtg(c: CardId, duel: &Duel) -> bool {
    c.is_faceup(duel) && c.is_set_card(duel, &[SET_ALTERGEIST]) && !c.is_code(duel, ID)
}
