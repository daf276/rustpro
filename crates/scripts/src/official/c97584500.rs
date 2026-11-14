// Ghostrick Mummy
use super::c36239585::RESET_FLAG_STANDARD;
use super::c80885284::{posop, postg, sumcon};
use crate::prelude::*;
use crate::testing::{Check, HAND, MZONE};

const ID: u32 = 97584500;

pub const LISTED_SERIES: &[u16] = &[SET_GHOSTRICK];

const GHOSTRICK_JIANGSHI: u32 = 80885284;
const GHOSTRICK_FAIRY: u32 = 36239585;

// Copies on the field, and Ghostrick monsters in the hand to Normal Summon in addition.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (ID, MZONE),
    (GHOSTRICK_JIANGSHI, MZONE),
    (GHOSTRICK_JIANGSHI, HAND),
    (GHOSTRICK_FAIRY, HAND),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_code(duel, EFFECT_CANNOT_SUMMON);
    e1.set_condition(duel, sumcon);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 0));
    e2.set_category(duel, CATEGORY_POSITION + CATEGORY_SET);
    e2.set_type(duel, EFFECT_TYPE_IGNITION);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_target(duel, postg::<ID, RESET_FLAG_STANDARD>);
    e2.set_operation(duel, posop);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 1));
    e3.set_type(duel, EFFECT_TYPE_FIELD);
    e3.set_code(duel, EFFECT_EXTRA_SUMMON_COUNT);
    e3.set_range(duel, LOCATION_MZONE);
    e3.set_target_range(duel, LOCATION_HAND | LOCATION_MZONE, 0);
    e3.set_target_card(duel, |duel, _, c| c.is_set_card(duel, &[SET_GHOSTRICK]));
    c.register_effect(duel, e3);
    let e4 = duel.create_effect(c);
    e4.set_type(duel, EFFECT_TYPE_FIELD);
    e4.set_property(duel, EFFECT_FLAG_PLAYER_TARGET);
    e4.set_code(duel, EFFECT_CANNOT_SPECIAL_SUMMON);
    e4.set_range(duel, LOCATION_MZONE);
    e4.set_target_range(duel, 1, 0);
    e4.set_target_card(duel, |duel, _, c| c.is_attribute_except(duel, ATTRIBUTE_DARK));
    c.register_effect(duel, e4);
}
