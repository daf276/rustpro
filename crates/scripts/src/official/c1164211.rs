// Shield of the Millennium Dynasty
use super::c38775407::add_placement_effects;
use crate::prelude::*;
use crate::testing::{Check, DECK, HAND};

const ID: u32 = 1164211;

pub const LISTED_SERIES: &[u16] = &[SET_MILLENNIUM];
pub const LISTED_NAMES: &[u32] = &[CARD_MILLENNIUM_CROSS];
// In the hand to place and Special Summon, Millennium Ankh in the hand to reveal and in the Deck to add.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (CARD_MILLENNIUM_CROSS, HAND),
    (CARD_MILLENNIUM_CROSS, DECK),
    (CARD_MILLENNIUM_CROSS, DECK),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_code(duel, EFFECT_INDESTRUCTABLE_EFFECT);
    e1.set_property(duel, EFFECT_FLAG_SINGLE_RANGE);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_value(duel, Value::EffectPlayer(|duel, _, re, _| re.is_spell_trap_effect(duel)));
    c.register_effect(duel, e1);
    add_placement_effects::<ID>(duel, c);
}
