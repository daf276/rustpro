// Barrier of the Voiceless Voice
use super::c25801745::is_lo_faceup;
use crate::prelude::*;
use crate::testing::{Check, DECK, HAND};

const ID: u32 = 98477480;

pub const LISTED_NAMES: &[u32] = &[25801745, ID];
pub const LISTED_SERIES: &[u16] = &[SET_VOICELESS_VOICE, SET_SKULL_GUARDIAN];

// In the hand, with Voiceless Voice cards in the Deck to search.
pub const SUPPORT: &[(u32, u16)] = &[(ID, HAND), (25801745, DECK), (10774240, DECK), (51296484, DECK)];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_FIELD);
    e2.set_code(duel, EFFECT_CANNOT_SELECT_BATTLE_TARGET);
    e2.set_range(duel, LOCATION_SZONE);
    e2.set_target_range(duel, 0, LOCATION_MZONE);
    e2.set_condition(duel, effcon);
    e2.set_value(duel, Value::Card(|duel, _, c| i32::from(!c.is_ritual_monster(duel))));
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_type(duel, EFFECT_TYPE_FIELD);
    e3.set_property(duel, EFFECT_FLAG_IGNORE_IMMUNE);
    e3.set_code(duel, EFFECT_CANNOT_BE_EFFECT_TARGET);
    e3.set_range(duel, LOCATION_SZONE);
    e3.set_target_range(duel, LOCATION_MZONE, 0);
    e3.set_condition(duel, effcon);
    e3.set_target_card(duel, |duel, _, c| c.is_attribute(duel, ATTRIBUTE_LIGHT));
    e3.set_value(duel, Value::EffectPlayer(aux::tgoval));
    c.register_effect(duel, e3);
    let e4 = duel.create_effect(c);
    e4.set_description(duel, aux::stringid(ID, 0));
    e4.set_category(duel, CATEGORY_TOHAND + CATEGORY_SEARCH);
    e4.set_type(duel, EFFECT_TYPE_IGNITION);
    e4.set_range(duel, LOCATION_SZONE);
    e4.set_count_limit(duel, 1, ID);
    e4.set_target(duel, thtg);
    e4.set_operation(duel, thop);
    c.register_effect(duel, e4);
}

fn ritfilter(c: CardId, duel: &Duel) -> bool {
    c.is_attribute(duel, ATTRIBUTE_LIGHT) && c.is_ritual_monster(duel) && c.is_faceup(duel)
}

fn effcon(duel: &Duel, a: &Args) -> bool {
    let tp = a.e.get_handler_player(duel);
    is_lo_faceup(duel, tp, LOCATION_ONFIELD) && duel.is_existing_matching_card(ritfilter, tp, LOCATION_MZONE, 0, 1, None)
}

fn thfilter(c: CardId, duel: &Duel) -> bool {
    (c.is_set_card(duel, &[SET_VOICELESS_VOICE]) || (c.is_set_card(duel, &[SET_SKULL_GUARDIAN]) && c.is_ritual_monster(duel)))
        && !c.is_code(duel, ID)
        && c.is_able_to_hand(duel)
}

fn thtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    stock::exists_target(duel, a, chk, thfilter, LOCATION_DECK, CATEGORY_TOHAND)
}

fn thop(duel: &mut Duel, a: &Args) -> Script<()> {
    stock::search(duel, a, thfilter, LOCATION_DECK)
}
