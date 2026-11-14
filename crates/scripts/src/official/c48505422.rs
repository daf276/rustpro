// Legendary Six Samurai - Shinai
use super::c74094021::spcon;
use super::c83039729::{targets_to_hand, thop};
use crate::prelude::*;
use crate::testing::{Check, GRAVE, HAND, MZONE};

const ID: u32 = 48505422;
const MIZUHO: u32 = 74094021;

pub const LISTED_SERIES: &[u16] = &[SET_SIX_SAMURAI];
pub const LISTED_NAMES: &[u32] = &[MIZUHO];
// In the hand with Mizuho on the field to summon it, which Tributes it as a cost, and Six Samurai (Kageki) in the Graveyard
// to add.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, MZONE),
    (MIZUHO, MZONE),
    (MIZUHO, MZONE),
    (2511717, GRAVE),
    (44686185, GRAVE),
];
pub const CHECKS: &[Check] = &[Check::SpecialSummoned, Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_code(duel, EFFECT_SPSUMMON_PROC);
    e1.set_property(duel, EFFECT_FLAG_UNCOPYABLE);
    e1.set_range(duel, LOCATION_HAND);
    e1.set_condition(duel, spcon::<MIZUHO>);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 0));
    e2.set_category(duel, CATEGORY_TOHAND);
    e2.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e2.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_F);
    e2.set_code(duel, EVENT_RELEASE);
    e2.set_condition(duel, |duel, a| a.e.get_handler(duel).is_previous_location(duel, LOCATION_ONFIELD));
    e2.set_target(duel, |duel, a, chk, chkc| targets_to_hand(duel, a, chk, chkc, filter));
    e2.set_operation(duel, thop);
    c.register_effect(duel, e2);
}

fn filter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_SIX_SAMURAI]) && c.get_code(duel) != ID && c.is_monster(duel) && c.is_able_to_hand(duel)
}
