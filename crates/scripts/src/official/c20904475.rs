// Kairo Ryu-Ge Emva
use super::c55154344::to_deck_cost;
use super::c56499179::{tdfilter, thgatefilter};
use crate::prelude::*;
use crate::testing::{Check, DECK, GRAVE, HAND};

const ID: u32 = 20904475;
const SEA_SPIRES: u32 = 28669235;

pub const LISTED_NAMES: &[u32] = &[SEA_SPIRES, ID];
// Ryu-Ge Realm - Sea Spires in the Deck to search and in the hand to activate and return as the cost, a copy in the
// Graveyard, and Foolish Burials to send monsters there from the Deck.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, GRAVE),
    (SEA_SPIRES, DECK),
    (SEA_SPIRES, DECK),
    (SEA_SPIRES, HAND),
    (81439173, HAND),
    (81439173, HAND),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
    Check::ResolvesEffect(aux::stringid(ID, 3)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_TOHAND + CATEGORY_SEARCH);
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_range(duel, LOCATION_HAND);
    e1.set_count_limit(duel, 1, ID);
    e1.set_cost(duel, Cost::self_discard);
    e1.set_target(duel, thtg);
    e1.set_operation(duel, thop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e2.set_type(duel, EFFECT_TYPE_IGNITION);
    e2.set_range(duel, LOCATION_GRAVE);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_condition(duel, |duel, _| duel.has_flag_effect_full(0, ID, 2));
    e2.set_target(duel, stock::special_summon_self_target);
    e2.set_operation(duel, stock::special_summon_self::<POS_FACEUP>);
    c.register_effect(duel, e2);
    aux::global_check(duel, ID, |duel| {
        let ge1 = duel.create_effect(c);
        ge1.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_CONTINUOUS);
        ge1.set_code(duel, EVENT_TO_GRAVE);
        ge1.set_operation(duel, checkop);
        duel.register_effect(ge1, 0);
    });
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 3));
    e3.set_category(duel, CATEGORY_REMOVE + CATEGORY_DRAW);
    e3.set_type(duel, EFFECT_TYPE_IGNITION);
    e3.set_range(duel, LOCATION_MZONE);
    e3.set_count_limit(duel, 1, (ID, 2));
    e3.set_cost(duel, |duel, a, chk| to_deck_cost(duel, a, chk, tdfilter::<SEA_SPIRES>));
    e3.set_target(duel, rmtg);
    e3.set_operation(duel, rmop);
    c.register_effect(duel, e3);
}

fn checkop(duel: &mut Duel, a: &Args) -> Script<()> {
    for tc in a.eg.unwrap_or_default() {
        if tc.is_monster(duel) && tc.is_previous_location(duel, LOCATION_HAND | LOCATION_DECK) && !tc.is_code(duel, ID) {
            duel.register_flag_effect(0, ID, RESET_PHASE | PHASE_END, 0, 1);
        }
    }
    Ok(())
}

fn thtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    stock::exists_target(duel, a, chk, thgatefilter::<SEA_SPIRES>, LOCATION_DECK, CATEGORY_TOHAND)
}

fn thop(duel: &mut Duel, a: &Args) -> Script<()> {
    stock::search(duel, a, thgatefilter::<SEA_SPIRES>, LOCATION_DECK)
}

fn rmtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    let ct = duel.get_matching_group_count(CardId::is_able_to_remove, tp, 0, LOCATION_HAND, None);
    if chk == 0 {
        return Ok(ct > 0 && duel.is_player_can_draw(1 - tp, ct));
    }
    duel.set_operation_info(0, CATEGORY_REMOVE, None, ct as u8, 1 - tp, LOCATION_HAND as i32);
    duel.set_operation_info(0, CATEGORY_DRAW, None, 0, 1 - tp, ct as i32);
    Ok(false)
}

fn rmop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    let g = duel.get_field_group(1 - tp, LOCATION_HAND, 0);
    if g.is_empty() {
        return Ok(());
    }
    let ct = duel.remove(g, POS_FACEUP, REASON_EFFECT)?;
    if ct > 0 {
        duel.draw(1 - tp, ct as u32, REASON_EFFECT)?;
    }
    Ok(())
}
