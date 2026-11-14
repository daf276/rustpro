// Ryu-Ge End
use crate::prelude::*;
use crate::testing::{Check, DECK, HAND, OPPONENT, SZONE};

const ID: u32 = 90664684;

pub const LISTED_SERIES: &[u16] = &[SET_RYU_GE];

// Set copies, Ryu-Ge Rivalry in the Deck to Set, and the opponent's Harpie's Feather Duster and Mystical Space Typhoon
// to destroy them.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, SZONE),
    (ID, SZONE),
    (81549048, DECK),
    (81549048, DECK),
    (18144506, HAND | OPPONENT),
    (5318639, HAND | OPPONENT),
];
// The Ryu-Ge Pendulum Monsters the destroying effect needs are batch 735's, not merged yet.
pub const CHECKS: &[Check] = &[
    Check::Resolves,
    Check::Report(&Check::ResolvesEffect(aux::stringid(ID, 0))),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e0 = duel.create_effect(c);
    e0.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e0.set_code(duel, EVENT_FREE_CHAIN);
    c.register_effect(duel, e0);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_property(duel, EFFECT_FLAG_IGNORE_IMMUNE);
    e1.set_code(duel, EFFECT_CANNOT_BE_EFFECT_TARGET);
    e1.set_range(duel, LOCATION_SZONE);
    e1.set_target_range(duel, LOCATION_MZONE, 0);
    e1.set_target_card(duel, |duel, _, c| c.is_set_card(duel, &[SET_RYU_GE]));
    e1.set_value(duel, Value::EffectPlayer(aux::tgoval));
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 0));
    e2.set_category(duel, CATEGORY_DESTROY);
    e2.set_type(duel, EFFECT_TYPE_QUICK_O);
    e2.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e2.set_code(duel, EVENT_FREE_CHAIN);
    e2.set_range(duel, LOCATION_SZONE);
    e2.set_hint_timing(duel, 0, TIMING_STANDBY_PHASE | TIMING_MAIN_END | TIMINGS_CHECK_MONSTER_E);
    e2.set_count_limit(duel, 1, ID);
    e2.set_condition(duel, |duel, a| {
        duel.is_existing_matching_card(desconfilter, a.tp, LOCATION_MZONE | LOCATION_EXTRA, 0, 1, None)
    });
    e2.set_cost(duel, descost);
    e2.set_target(duel, destg);
    e2.set_operation(duel, desop);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 1));
    e3.set_category(duel, CATEGORY_SET);
    e3.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e3.set_property(duel, EFFECT_FLAG_DELAY);
    e3.set_code(duel, EVENT_DESTROYED);
    e3.set_count_limit(duel, 1, (ID, 1));
    e3.set_target(duel, settg);
    e3.set_operation(duel, setop);
    c.register_effect(duel, e3);
}

fn desconfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_RYU_GE]) && c.is_type(duel, TYPE_PENDULUM) && c.is_faceup(duel)
}

fn tdfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_RYU_GE])
        && c.is_monster(duel)
        && (c.is_location(duel, LOCATION_HAND) || c.is_faceup(duel))
        && c.is_able_to_deck_as_cost(duel)
}

fn descost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let tp = a.tp;
    let location = LOCATION_HAND | LOCATION_GRAVE | LOCATION_REMOVED;
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(tdfilter, tp, location, 0, 2, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TODECK);
    let g = duel.select_matching_card(tp, tdfilter, tp, location, 0, 2, 2, None)?;
    let (rvg, hintg) = g.split(duel, |c, duel| c.is_location(duel, LOCATION_HAND), None);
    if !rvg.is_empty() {
        duel.confirm_cards(1 - tp, rvg)?;
    }
    if !hintg.is_empty() {
        duel.hint_selection(hintg);
    }
    duel.send_to_deck(g, PLAYER_NONE, SEQ_DECKSHUFFLE, REASON_COST)?;
    Ok(false)
}

fn destg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_MZONE));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(aux::TRUE, tp, LOCATION_MZONE, LOCATION_MZONE, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_DESTROY);
    let g = duel.select_target(tp, aux::TRUE, tp, LOCATION_MZONE, LOCATION_MZONE, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_DESTROY, g, 1, tp, 0);
    Ok(false)
}

fn desop(duel: &mut Duel, a: &Args) -> Script<()> {
    if let Some(tc) = duel.get_first_target().filter(|tc| tc.is_relate_to_effect(duel, a.e)) {
        duel.destroy(tc, REASON_EFFECT)?;
    }
    Ok(())
}

fn setfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_RYU_GE])
        && c.is_trap(duel)
        && !c.is_continuous_trap(duel)
        && c.is_ssetable(duel)
        && (c.is_faceup(duel) || !c.is_location(duel, LOCATION_REMOVED))
}

fn settg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(setfilter, tp, LOCATION_DECK | LOCATION_GRAVE | LOCATION_REMOVED, 0, 1, None));
    }
    duel.set_possible_operation_info(0, CATEGORY_LEAVE_GRAVE, None, 1, tp, 0);
    Ok(false)
}

fn setop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SET);
    let location = LOCATION_DECK | LOCATION_GRAVE | LOCATION_REMOVED;
    let g = duel.select_matching_card(tp, aux::necro_valley_filter(setfilter), tp, location, 0, 1, 1, None)?;
    if !g.is_empty() {
        duel.sset(tp, g)?;
    }
    Ok(())
}
