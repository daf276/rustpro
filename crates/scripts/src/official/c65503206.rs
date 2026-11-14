// Scrap Soldier
use super::c19139516::{desop, destg, regop, thcon, thop, thtg};
use crate::prelude::*;
use crate::testing::{Check, GRAVE, MZONE, OPPONENT};

const ID: u32 = 65503206;

pub const LISTED_SERIES: &[u16] = &[SET_SCRAP];
pub const LISTED_NAMES: &[u32] = &[ID];
// On the field in Defense Position against Skull Servants too weak to destroy it, with Scrap Raptor to destroy it and Scrap monsters in the Graveyard.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (ID, MZONE),
    (ID, MZONE),
    (ID, MZONE),
    (80529459, MZONE),
    (16449363, GRAVE),
    (4334811, GRAVE),
    (4334811, GRAVE),
    (32274490, MZONE | OPPONENT),
    (32274490, MZONE | OPPONENT),
    (32274490, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_CONTINUOUS);
    e1.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE);
    e1.set_code(duel, EVENT_BE_BATTLE_TARGET);
    e1.set_operation(duel, regop::<ID>);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 0));
    e2.set_category(duel, CATEGORY_DESTROY);
    e2.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_F);
    e2.set_code(duel, EVENT_PHASE | PHASE_BATTLE);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_count_limit(duel, 1, 0);
    e2.set_target(duel, destg::<ID>);
    e2.set_operation(duel, desop);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 1));
    e3.set_category(duel, CATEGORY_TOHAND);
    e3.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e3.set_property(duel, EFFECT_FLAG_CARD_TARGET + EFFECT_FLAG_DELAY);
    e3.set_code(duel, EVENT_DESTROYED);
    e3.set_condition(duel, thcon);
    e3.set_target(duel, thtg::<ID>);
    e3.set_operation(duel, thop);
    c.register_effect(duel, e3);
    let e4 = duel.create_effect(c);
    e4.set_type(duel, EFFECT_TYPE_SINGLE);
    e4.set_code(duel, EFFECT_CANNOT_BE_SYNCHRO_MATERIAL);
    e4.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE + EFFECT_FLAG_UNCOPYABLE);
    e4.set_value(duel, Value::Card(|duel, _, c| i32::from(!c.is_set_card(duel, &[SET_SCRAP]))));
    c.register_effect(duel, e4);
}
