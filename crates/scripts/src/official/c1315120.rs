// T.G. Striker
use crate::prelude::*;
use crate::testing::{Check, DECK, HAND, MZONE, OPPONENT};

const ID: u32 = 1315120;

pub const LISTED_SERIES: &[u16] = &[SET_TG];
// In the hand to Special Summon onto an empty field against Sangan, and on the field to be destroyed by the opponent's Raigeki
// and search another T.G. monster from the Deck.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, MZONE),
    (94350039, DECK),
    (94350039, DECK),
    (26202165, MZONE | OPPONENT),
    (12580477, HAND | OPPONENT),
];
pub const CHECKS: &[Check] = &[Check::SpecialSummoned, Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_code(duel, EFFECT_SPSUMMON_PROC);
    e1.set_property(duel, EFFECT_FLAG_UNCOPYABLE);
    e1.set_range(duel, LOCATION_HAND);
    e1.set_condition(duel, spcon);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_CONTINUOUS);
    e2.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE);
    e2.set_code(duel, EVENT_TO_GRAVE);
    e2.set_operation(duel, regop::<ID, 0>);
    c.register_effect(duel, e2);
}

fn spcon(duel: &Duel, a: &Args) -> bool {
    let Some(c) = a.c else { return true };
    let tp = c.get_controler(duel);
    duel.get_field_group_count(tp, LOCATION_MZONE, 0) == 0
        && duel.get_field_group_count(tp, 0, LOCATION_MZONE) > 0
        && duel.get_location_count(tp, LOCATION_MZONE) > 0
}

pub fn regop<const CODE: u32, const DESCRIPTION: u64>(duel: &mut Duel, a: &Args) -> Script<()> {
    let c = a.e.get_handler(duel);
    if c.get_previous_location(duel) & LOCATION_ONFIELD != 0 && c.is_reason(duel, REASON_DESTROY) {
        let e1 = duel.create_effect(c);
        e1.set_description(duel, aux::stringid(CODE, DESCRIPTION));
        e1.set_category(duel, CATEGORY_TOHAND + CATEGORY_SEARCH);
        e1.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
        e1.set_code(duel, EVENT_PHASE + PHASE_END);
        e1.set_count_limit(duel, 1, 0);
        e1.set_range(duel, LOCATION_GRAVE);
        e1.set_target(duel, thtg::<CODE>);
        e1.set_operation(duel, thop::<CODE>);
        e1.set_reset(duel, RESETS_STANDARD_PHASE_END);
        c.register_effect(duel, e1);
    }
    Ok(())
}

fn filter<const CODE: u32>(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_TG]) && c.get_code(duel) != CODE && c.is_monster(duel) && c.is_able_to_hand(duel)
}

fn thtg<const CODE: u32>(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    stock::exists_target(duel, a, chk, filter::<CODE>, LOCATION_DECK, CATEGORY_TOHAND)
}

fn thop<const CODE: u32>(duel: &mut Duel, a: &Args) -> Script<()> {
    stock::search(duel, a, filter::<CODE>, LOCATION_DECK)
}
