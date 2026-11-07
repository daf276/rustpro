// Droll & Lock Bird
use crate::prelude::*;
use crate::testing::{HAND, OPPONENT};

const ID: u32 = 94145021;

// A copy in the hand, and Pot of Greed for the opponent to draw outside the Draw Phase.
pub const SUPPORT: &[(u32, u16)] = &[(ID, HAND), (55144522, HAND | OPPONENT)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_DISABLE);
    e1.set_property(duel, EFFECT_FLAG_DELAY);
    e1.set_type(duel, EFFECT_TYPE_QUICK_O);
    e1.set_code(duel, EVENT_CUSTOM + ID);
    e1.set_range(duel, LOCATION_HAND);
    e1.set_condition(duel, condition);
    e1.set_cost(duel, cost);
    e1.set_operation(duel, operation);
    c.register_effect(duel, e1);
    aux::global_check(duel, ID, |duel| {
        let ge1 = duel.create_effect(c);
        ge1.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_CONTINUOUS);
        ge1.set_code(duel, EVENT_TO_HAND);
        ge1.set_condition(duel, regcon);
        ge1.set_operation(duel, regop);
        duel.register_effect(ge1, 0);
    });
}

fn cfilter(c: CardId, duel: &Duel, tp: u8) -> bool {
    c.is_controler(duel, tp) && c.is_previous_location(duel, LOCATION_DECK)
}

// The players whose hand got cards from the Deck, which regcon keeps in its label for regop; regop recomputes it from
// the same event instead, since a condition cannot set a label here.
fn event_player(duel: &Duel, eg: Option<CardSet>) -> Option<u8> {
    let eg = eg.unwrap_or_default();
    match (
        eg.is_exists(duel, |c, duel| cfilter(c, duel, 0), 1, None),
        eg.is_exists(duel, |c, duel| cfilter(c, duel, 1), 1, None),
    ) {
        (false, false) => None,
        (true, false) => Some(0),
        (false, true) => Some(1),
        (true, true) => Some(PLAYER_ALL),
    }
}

fn regcon(duel: &Duel, a: &Args) -> bool {
    if duel.is_phase(PHASE_DRAW) || duel.is_phase(PHASE_DAMAGE) {
        return false;
    }
    event_player(duel, a.eg).is_some()
}

fn regop(duel: &mut Duel, a: &Args) -> Script<()> {
    let Some(player) = event_player(duel, a.eg) else { return Ok(()) };
    duel.raise_event(a.eg.unwrap_or_default(), EVENT_CUSTOM + ID, a.re, a.r, a.rp, a.ep, player.into())
}

fn condition(_: &Duel, a: &Args) -> bool {
    a.ev == u32::from(1 - a.tp) || a.ev == u32::from(PLAYER_ALL)
}

fn cost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let c = a.e.get_handler(duel);
    if chk == 0 {
        return Ok(c.is_location(duel, LOCATION_HAND) && c.is_able_to_grave_as_cost(duel));
    }
    duel.send_to_grave(c, REASON_COST)?;
    Ok(false)
}

fn operation(duel: &mut Duel, a: &Args) -> Script<()> {
    let (c, tp) = (a.e.get_handler(duel), a.tp);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_code(duel, EFFECT_CANNOT_TO_HAND);
    e1.set_target_range(duel, LOCATION_DECK, LOCATION_DECK);
    e1.set_reset(duel, RESET_PHASE | PHASE_END);
    duel.register_effect(e1, tp);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_FIELD);
    e2.set_property(duel, EFFECT_FLAG_PLAYER_TARGET);
    e2.set_code(duel, EFFECT_CANNOT_DRAW);
    e2.set_reset(duel, RESET_PHASE | PHASE_END);
    e2.set_target_range(duel, 1, 1);
    duel.register_effect(e2, tp);
    let e3 = duel.create_effect(c);
    e3.set_property(duel, EFFECT_FLAG_PLAYER_TARGET + EFFECT_FLAG_CLIENT_HINT);
    e3.set_description(duel, aux::stringid(ID, 1));
    e3.set_reset(duel, RESET_PHASE | PHASE_END);
    e3.set_target_range(duel, 1, 1);
    duel.register_effect(e3, tp);
    Ok(())
}
