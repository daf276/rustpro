// Blitzclique Alternator
use super::c84401954::thfilter as blitzthfilter;
use crate::prelude::*;
use crate::testing::{Check, DECK, GRAVE, HAND};

const ID: u32 = 59921227;

pub const LISTED_NAMES: &[u32] = &[CARD_HIDEOUT_IN_THE_SKY_COULOMB];
// A copy in the GY, Surge Blitzclique in the hand to shuffle, and Thunder Normal Monsters (LaLa Li-oon) and a Blitzclique Spell
// (Blitzclique - Breakaway) in the Deck to search.
pub const SUPPORT: &[(u32, u16)] = &[(ID, GRAVE), (22912101, HAND), (9430387, DECK), (64049762, DECK)];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e0 = duel.create_effect(c);
    e0.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e0.set_code(duel, EVENT_FREE_CHAIN);
    e0.set_property(duel, EFFECT_FLAG_DAMAGE_STEP);
    e0.set_hint_timing(duel, TIMING_DAMAGE_STEP, TIMING_DAMAGE_STEP);
    e0.set_condition(duel, aux::stat_change_damage_step_condition);
    c.register_effect(duel, e0);
    let e1a = duel.create_effect(c);
    e1a.set_type(duel, EFFECT_TYPE_FIELD);
    e1a.set_code(duel, EFFECT_UPDATE_ATTACK);
    e1a.set_range(duel, LOCATION_SZONE);
    e1a.set_target_range(duel, 0, LOCATION_MZONE);
    e1a.set_value(
        duel,
        Value::Card(|duel, e, _| {
            let thunder = aux::faceup_filter(|c, duel| c.is_race(duel, RACE_THUNDER));
            -300 * duel.get_matching_group_count(thunder, e.get_handler_player(duel), LOCATION_MZONE, 0, None) as i32
        }),
    );
    c.register_effect(duel, e1a);
    let e1b = e1a.clone_effect(duel);
    e1b.set_code(duel, EFFECT_UPDATE_DEFENSE);
    c.register_effect(duel, e1b);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 0));
    e2.set_category(duel, CATEGORY_TOHAND + CATEGORY_SEARCH);
    e2.set_type(duel, EFFECT_TYPE_QUICK_O);
    e2.set_code(duel, EVENT_FREE_CHAIN);
    e2.set_range(duel, LOCATION_SZONE);
    e2.set_count_limit(duel, 1, (ID, 0));
    e2.set_cost(duel, thunderthcost);
    e2.set_target(duel, thunderthtg);
    e2.set_operation(duel, thunderthop);
    e2.set_hint_timing(duel, 0, TIMING_STANDBY_PHASE | TIMING_MAIN_END | TIMINGS_CHECK_MONSTER_E);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 1));
    e3.set_category(duel, CATEGORY_TOHAND + CATEGORY_SEARCH);
    e3.set_type(duel, EFFECT_TYPE_QUICK_O);
    e3.set_code(duel, EVENT_FREE_CHAIN);
    e3.set_range(duel, LOCATION_GRAVE);
    e3.set_count_limit(duel, 1, (ID, 1));
    e3.set_cost(duel, Cost::self_banish);
    e3.set_target(duel, blitzthtg);
    e3.set_operation(duel, blitzthop);
    e3.set_hint_timing(duel, 0, TIMING_STANDBY_PHASE | TIMING_MAIN_END | TIMINGS_CHECK_MONSTER_E);
    c.register_effect(duel, e3);
}

fn thunderthfilter(c: CardId, duel: &Duel, code: u32) -> bool {
    c.is_race(duel, RACE_THUNDER) && !c.is_code(duel, code) && c.is_able_to_hand(duel)
}

fn thunderthcostfilter(c: CardId, duel: &Duel, tp: u8) -> bool {
    let code = c.get_code(duel);
    c.is_set_card(duel, &[SET_BLITZCLIQUE])
        && c.is_able_to_deck_as_cost(duel)
        && (c.is_location(duel, LOCATION_HAND) || c.is_faceup(duel))
        && duel.is_existing_matching_card(|c, duel| thunderthfilter(c, duel, code), tp, LOCATION_DECK, 0, 1, None)
}

fn thunderthcost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    let filter = |sc: CardId, duel: &Duel| thunderthcostfilter(sc, duel, tp);
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(filter, tp, LOCATION_HAND | LOCATION_ONFIELD, 0, 1, Some(c.into())));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TODECK);
    let g = duel.select_matching_card(tp, filter, tp, LOCATION_HAND | LOCATION_ONFIELD, 0, 1, 1, Some(c.into()))?;
    let Some(sc) = g.get_first() else { return Ok(false) };
    if sc.is_location(duel, LOCATION_HAND) {
        duel.confirm_cards(1 - tp, sc)?;
    } else {
        duel.hint_selection(sc);
    }
    duel.send_to_deck(sc, PLAYER_NONE, SEQ_DECKSHUFFLE, REASON_COST)?;
    e.set_label(duel, i64::from(sc.get_code(duel)));
    Ok(false)
}

fn thunderthtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(true);
    }
    duel.set_operation_info(0, CATEGORY_TOHAND, None, 1, a.tp, LOCATION_DECK as i32);
    Ok(false)
}

fn thunderthop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    let code = a.e.get_label(duel) as u32;
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_ATOHAND);
    let g = duel.select_matching_card(tp, |c, duel| thunderthfilter(c, duel, code), tp, LOCATION_DECK, 0, 1, 1, None)?;
    if !g.is_empty() {
        duel.send_to_hand(g, PLAYER_NONE, REASON_EFFECT)?;
        duel.confirm_cards(1 - tp, g)?;
    }
    Ok(())
}

fn blitzthtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(blitzthfilter, a.tp, LOCATION_DECK, 0, 1, None));
    }
    duel.set_operation_info(0, CATEGORY_TOHAND, None, 1, a.tp, LOCATION_DECK as i32);
    Ok(false)
}

fn blitzthop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_ATOHAND);
    let g = duel.select_matching_card(tp, blitzthfilter, tp, LOCATION_DECK, 0, 1, 1, None)?;
    if !g.is_empty() {
        duel.send_to_hand(g, PLAYER_NONE, REASON_EFFECT)?;
        duel.confirm_cards(1 - tp, g)?;
    }
    Ok(())
}
