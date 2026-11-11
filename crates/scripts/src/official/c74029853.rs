// Golden Bamboo Sword
use crate::prelude::*;
use crate::testing::{Check, HAND, MZONE};

const ID: u32 = 74029853;
const BROKEN_BAMBOO_SWORD: u32 = 41587307;

// In the hand, with Broken Bamboo Sword in the hand to equip to a monster, which is its condition.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (BROKEN_BAMBOO_SWORD, HAND),
    (BROKEN_BAMBOO_SWORD, HAND),
    (4035199, MZONE),
    (4035199, MZONE),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(0)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_category(duel, CATEGORY_DRAW);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_property(duel, EFFECT_FLAG_PLAYER_TARGET);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_condition(duel, condition);
    e1.set_target(duel, stock::draw_target::<2>);
    e1.set_operation(duel, stock::draw);
    c.register_effect(duel, e1);
}

fn cfilter(c: CardId, duel: &Duel) -> bool {
    c.is_faceup(duel) && c.is_set_card(duel, &[SET_BAMBOO_SWORD]) && c.is_type(duel, TYPE_EQUIP)
}

fn condition(duel: &Duel, a: &Args) -> bool {
    duel.is_existing_matching_card(cfilter, a.tp, LOCATION_ONFIELD, 0, 1, None)
}
