// Madolche Chouxvalier
use super::c74641045::{retcon, retop, rettg};
use crate::prelude::*;
use crate::testing::{HAND, MZONE, OPPONENT};

const ID: u32 = 75363626;

pub const LISTED_SERIES: &[u16] = &[SET_MADOLCHE];
// On the field with other Madolche monsters, and the opponent's Raigeki to destroy it and monsters to attack.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (74641045, MZONE),
    (12580477, HAND | OPPONENT),
    (26202165, MZONE | OPPONENT),
    (13597785, MZONE | OPPONENT),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_TODECK);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_F);
    e1.set_code(duel, EVENT_TO_GRAVE);
    e1.set_condition(duel, retcon);
    e1.set_target(duel, rettg);
    e1.set_operation(duel, retop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_FIELD);
    e2.set_code(duel, EFFECT_CANNOT_SELECT_BATTLE_TARGET);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_target_range(duel, 0, LOCATION_MZONE);
    e2.set_value(duel, Value::Card(|duel, _, c| i32::from(atktg(c, duel))));
    c.register_effect(duel, e2);
}

fn atktg(c: CardId, duel: &Duel) -> bool {
    c.is_faceup(duel) && c.get_code(duel) != ID && c.is_set_card(duel, &[SET_MADOLCHE])
}
