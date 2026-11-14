// Koa'ki Meiru Drago
use engine::constants::MSG_CARD_SELECTED;

use crate::prelude::*;
use crate::testing::{Check, HAND, MZONE};

const ID: u32 = 12435193;

pub const LISTED_NAMES: &[u32] = &[36623431];

// On the field with Iron Core of Koa'ki Meiru in the hand for its upkeep.
pub const SUPPORT: &[(u32, u16)] = &[(ID, MZONE), (36623431, HAND)];
// Both effects are continuous: the upkeep shows as the card's selection hint.
pub const CHECKS: &[Check] = &[Check::Message(MSG_CARD_SELECTED)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_CONTINUOUS);
    e1.set_property(duel, EFFECT_FLAG_UNCOPYABLE + EFFECT_FLAG_CANNOT_DISABLE);
    e1.set_code(duel, EVENT_PHASE + PHASE_END);
    e1.set_count_limit(duel, 1, 0);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_condition(duel, |duel, a| duel.is_turn_player(a.tp));
    e1.set_operation(duel, |duel, a| super::c45041488::upkeep(duel, a, ID, RACE_DRAGON, false));
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_FIELD);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_code(duel, EFFECT_CANNOT_SPECIAL_SUMMON);
    e2.set_property(duel, EFFECT_FLAG_PLAYER_TARGET);
    e2.set_target_range(duel, 1, 1);
    e2.set_target_card(duel, |duel, _, c| c.is_attribute(duel, ATTRIBUTE_DARK + ATTRIBUTE_LIGHT));
    c.register_effect(duel, e2);
}
