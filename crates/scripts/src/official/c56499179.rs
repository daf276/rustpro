// Genro Ryu-Ge Hakva
use super::c55154344::to_deck_cost;
use crate::prelude::*;
use crate::testing::{Check, DECK, HAND};

const ID: u32 = 56499179;
const WYRM_WINDS: u32 = 55154344;

pub const LISTED_SERIES: &[u16] = &[SET_RYU_GE];
pub const LISTED_NAMES: &[u32] = &[WYRM_WINDS, ID];
// Ryu-Ge Realm - Wyrm Winds in the Deck to search and in the hand to activate and return as the cost.
pub const SUPPORT: &[(u32, u16)] = &[(ID, HAND), (WYRM_WINDS, DECK), (WYRM_WINDS, DECK), (WYRM_WINDS, HAND)];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
    Check::ResolvesEffect(aux::stringid(ID, 2)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_TOHAND + CATEGORY_SEARCH);
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_range(duel, LOCATION_HAND);
    e1.set_count_limit(duel, 1, ID);
    e1.set_cost(duel, Cost::self_banish);
    e1.set_target(duel, thgatetg);
    e1.set_operation(duel, thgateop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e2.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e2.set_property_full(duel, EFFECT_FLAG_DELAY, EFFECT_FLAG2_CHECK_SIMULTANEOUS);
    e2.set_code(duel, EVENT_REMOVE);
    e2.set_range(duel, LOCATION_REMOVED);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_condition(duel, |duel, a| {
        let eg = a.eg.unwrap_or_default();
        !eg.contains(a.e.get_handler(duel)) && eg.is_exists(duel, |c, duel| c.is_faceup(duel) && c.is_monster(duel), 1, None)
    });
    e2.set_target(duel, sptg);
    e2.set_operation(duel, stock::special_summon_self::<POS_FACEUP>);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 2));
    e3.set_category(duel, CATEGORY_TOHAND + CATEGORY_SEARCH);
    e3.set_type(duel, EFFECT_TYPE_IGNITION);
    e3.set_range(duel, LOCATION_MZONE);
    e3.set_count_limit(duel, 1, (ID, 2));
    e3.set_cost(duel, |duel, a, chk| to_deck_cost(duel, a, chk, tdfilter::<WYRM_WINDS>));
    e3.set_target(duel, thtg);
    e3.set_operation(duel, thop);
    c.register_effect(duel, e3);
}

pub fn thgatefilter<const CODE: u32>(c: CardId, duel: &Duel) -> bool {
    c.is_code(duel, CODE) && c.is_able_to_hand(duel)
}

fn thgatetg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    stock::exists_target(duel, a, chk, thgatefilter::<WYRM_WINDS>, LOCATION_DECK, CATEGORY_TOHAND)
}

fn thgateop(duel: &mut Duel, a: &Args) -> Script<()> {
    stock::search(duel, a, thgatefilter::<WYRM_WINDS>, LOCATION_DECK)
}

fn spconfilter(c: CardId, duel: &Duel) -> bool {
    c.is_monster(duel) && c.is_faceup(duel) && !c.is_code(duel, ID)
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if chk == 0 {
        return Ok(duel.get_location_count(tp, LOCATION_MZONE) > 0
            && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
            && duel.is_existing_matching_card(spconfilter, tp, LOCATION_REMOVED, LOCATION_REMOVED, 2, None));
    }
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, Some(c.into()), 1, tp, 0);
    Ok(false)
}

pub fn tdfilter<const CODE: u32>(c: CardId, duel: &Duel) -> bool {
    c.is_code(duel, CODE) && c.is_faceup(duel) && c.is_able_to_deck_as_cost(duel)
}

fn thfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_RYU_GE])
        && c.is_able_to_hand(duel)
        && !c.is_code(duel, ID)
        && (c.is_faceup(duel) || !c.is_location(duel, LOCATION_REMOVED))
}

fn thtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    stock::exists_target(
        duel,
        a,
        chk,
        thfilter,
        LOCATION_DECK | LOCATION_GRAVE | LOCATION_REMOVED,
        CATEGORY_TOHAND,
    )
}

fn thop(duel: &mut Duel, a: &Args) -> Script<()> {
    stock::search(
        duel,
        a,
        aux::necro_valley_filter(thfilter),
        LOCATION_DECK | LOCATION_GRAVE | LOCATION_REMOVED,
    )
}
