// Wattberyx
use super::c10209545::{hdcon, hdop, hdtg};
use crate::prelude::*;
use crate::testing::{Check, HAND, MZONE};

const ID: u32 = 5554990;

// Attacks directly after Raigeki clears the opponent's field.
pub const SUPPORT: &[(u32, u16)] = &[(ID, MZONE), (ID, MZONE), (ID, MZONE), (12580477, HAND)];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_HANDES);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_F);
    e1.set_code(duel, EVENT_BATTLE_DAMAGE);
    e1.set_condition(duel, hdcon);
    e1.set_target(duel, hdtg);
    e1.set_operation(duel, hdop);
    c.register_effect(duel, e1);
}
