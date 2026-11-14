// Gravekeeper's Trap
use crate::prelude::*;
use crate::testing::{Check, DECK, HAND, SZONE};

const ID: u32 = 98715423;

pub const LISTED_NAMES: &[u32] = &[CARD_EXCHANGE_SPIRIT];
pub const LISTED_SERIES: &[u16] = &[SET_GRAVEKEEPERS];

// Copies in the hand and face-down, with Gravekeeper's Spy and Mudora to search.
pub const SUPPORT: &[(u32, u16)] = &[(ID, HAND), (ID, SZONE), (24317029, DECK), (24317029, DECK), (82108372, DECK)];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_FIELD);
    e2.set_property(duel, EFFECT_FLAG_PLAYER_TARGET);
    e2.set_code(duel, EFFECT_CANNOT_ACTIVATE);
    e2.set_range(duel, LOCATION_SZONE);
    e2.set_target_range(duel, 0, 1);
    e2.set_condition(duel, limcon);
    let e3 = e2.clone_effect(duel);
    e2.set_value(
        duel,
        Value::EffectPlayer(|duel, _, re, _| re.get_activate_location(duel) == LOCATION_GRAVE),
    );
    c.register_effect(duel, e2);
    e3.set_code(duel, EFFECT_CANNOT_SPECIAL_SUMMON);
    e3.set_target_spsummon(duel, |duel, _, c, _, _, _, _, _| c.is_location(duel, LOCATION_GRAVE));
    c.register_effect(duel, e3);
    let e4 = duel.create_effect(c);
    e4.set_description(duel, aux::stringid(ID, 0));
    e4.set_category(duel, CATEGORY_SEARCH + CATEGORY_TOHAND);
    e4.set_type(duel, EFFECT_TYPE_QUICK_O);
    e4.set_code(duel, EVENT_FREE_CHAIN);
    e4.set_range(duel, LOCATION_SZONE);
    e4.set_count_limit(duel, 1, ID);
    e4.set_hint_timing(duel, 0, TIMINGS_CHECK_MONSTER | TIMING_MAIN_END);
    e4.set_condition(duel, |duel, _| duel.is_main_phase());
    e4.set_cost(duel, thcost);
    e4.set_target(duel, thtg);
    e4.set_operation(duel, thop);
    c.register_effect(duel, e4);
    let e5 = duel.create_effect(c);
    e5.set_description(duel, aux::stringid(ID, 1));
    e5.set_category(duel, CATEGORY_TOGRAVE);
    e5.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_F);
    e5.set_code(duel, EVENT_PREDRAW);
    e5.set_range(duel, LOCATION_SZONE);
    e5.set_count_limit(duel, 1, (ID, 1));
    e5.set_condition(duel, tgcon);
    e5.set_target(duel, tgtg);
    e5.set_operation(duel, tgop);
    c.register_effect(duel, e5);
    aux::global_check(duel, ID, |duel| {
        let ge1 = duel.create_effect(c);
        ge1.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_CONTINUOUS);
        ge1.set_code(duel, EVENT_DRAW);
        ge1.set_operation(duel, checkop);
        duel.register_effect(ge1, 0);
    });
}

fn limcon(duel: &Duel, a: &Args) -> bool {
    let player = a.e.get_handler_player(duel);
    duel.is_existing_matching_card(|c, duel| c.is_code(duel, CARD_EXCHANGE_SPIRIT), player, LOCATION_GRAVE, 0, 1, None)
}

fn thcost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(CardId::is_discardable, tp, LOCATION_HAND, 0, 1, None));
    }
    duel.discard_hand_full(tp, CardId::is_discardable, 1, 1, REASON_COST | REASON_DISCARD, None)?;
    Ok(false)
}

fn thfilter(c: CardId, duel: &Duel) -> bool {
    c.is_able_to_hand(duel)
        && c.is_monster(duel)
        && (c.is_set_card(duel, &[SET_GRAVEKEEPERS]) || (c.is_attribute(duel, ATTRIBUTE_EARTH) && c.is_race(duel, RACE_FAIRY)))
}

fn thtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    stock::exists_target(duel, a, chk, thfilter, LOCATION_DECK, CATEGORY_TOHAND)
}

fn thop(duel: &mut Duel, a: &Args) -> Script<()> {
    stock::search(duel, a, thfilter, LOCATION_DECK)
}

fn tgcon(duel: &Duel, a: &Args) -> bool {
    let opp = 1 - a.tp;
    duel.is_turn_player(opp)
        && duel.is_player_affected_by_effect(opp, EFFECT_CANNOT_DRAW).is_none()
        && duel.get_draw_count(opp) > 0
        && duel.get_flag_effect(opp, ID) == 0
}

fn tgtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(true);
    }
    duel.hint(HINT_SELECTMSG, a.tp, HINTMSG_CODE);
    let code = duel.announce_card(a.tp, &[u64::from(TYPE_EXTRA), OPCODE_ISTYPE, OPCODE_NOT])?;
    a.e.set_label(duel, i64::from(code));
    Ok(false)
}

fn tgop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if !c.is_relate_to_effect(duel, e) || duel.get_draw_count(1 - tp) < 1 {
        return Ok(());
    }
    let label = e.get_label(duel);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_CONTINUOUS);
    e1.set_code(duel, EVENT_DRAW);
    e1.set_count_limit(duel, 1, 0);
    e1.set_operation(duel, drawcheck);
    e1.set_label(duel, label);
    e1.set_reset(duel, RESET_PHASE | PHASE_DRAW);
    duel.register_effect(e1, tp);
    Ok(())
}

fn drawcheck(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    let Some(eg) = a.eg.filter(|eg| !eg.is_empty()) else {
        return Ok(());
    };
    if duel.is_phase(PHASE_DRAW) && (a.r & REASON_RULE) == REASON_RULE {
        duel.confirm_cards(tp, eg)?;
        let code = a.e.get_label(duel) as u32;
        let dg = eg.filter(duel, |c, duel| c.is_code(duel, code), None);
        if !dg.is_empty() {
            duel.send_to_grave(dg, REASON_EFFECT)?;
        }
        duel.shuffle_hand(1 - tp)?;
    }
    Ok(())
}

fn checkop(duel: &mut Duel, a: &Args) -> Script<()> {
    if duel.is_phase(PHASE_DRAW) && (a.r & REASON_RULE) == REASON_RULE {
        duel.register_flag_effect(a.ep, ID, RESET_PHASE | PHASE_STANDBY, 0, 1);
    }
    Ok(())
}
