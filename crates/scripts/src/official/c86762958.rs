// Assist★Yummy!
use crate::prelude::*;
use crate::testing::{Check, DECK, GRAVE, HAND, MZONE, SZONE};

const ID: u32 = 86762958;

pub const LISTED_NAMES: &[u32] = &[ID];
pub const LISTED_SERIES: &[u16] = &[SET_YUMMY];
// Set and in the Graveyard, with face-up Yummy monsters to send as the cost and Yummy cards in the Deck to send.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, SZONE),
    (ID, SZONE),
    (ID, HAND),
    (ID, GRAVE),
    (31425736, MZONE),
    (68810435, MZONE),
    (4215180, MZONE),
    (10966439, DECK),
    (31425736, DECK),
    (66975205, DECK),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
    Check::ResolvesEffect(aux::stringid(ID, 2)),
];

const TIMINGS: u32 = TIMING_MAIN_END | TIMING_BATTLE_START | TIMING_BATTLE_PHASE | TIMING_BATTLE_STEP_END | TIMING_BATTLE_END;

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e0 = duel.create_effect(c);
    e0.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e0.set_code(duel, EVENT_FREE_CHAIN);
    e0.set_hint_timing(duel, 0, TIMINGS);
    c.register_effect(duel, e0);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_RECOVER);
    e1.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e1.set_code(duel, EVENT_PHASE + PHASE_STANDBY);
    e1.set_range(duel, LOCATION_SZONE);
    e1.set_count_limit(duel, 1, 0);
    e1.set_target(duel, lptg);
    e1.set_operation(duel, lpop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_TOHAND);
    e2.set_type(duel, EFFECT_TYPE_QUICK_O);
    e2.set_code(duel, EVENT_FREE_CHAIN);
    e2.set_range(duel, LOCATION_SZONE);
    e2.set_count_limit(duel, 1, ID);
    e2.set_condition(duel, |duel, a| {
        duel.is_main_phase() || (duel.is_battle_phase() && duel.is_turn_player(1 - a.tp))
    });
    e2.set_cost(duel, retthcost);
    e2.set_target(duel, retthtg);
    e2.set_operation(duel, retthop);
    e2.set_hint_timing(duel, 0, TIMINGS);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 2));
    e3.set_category(duel, CATEGORY_TOGRAVE);
    e3.set_type(duel, EFFECT_TYPE_QUICK_O);
    e3.set_code(duel, EVENT_FREE_CHAIN);
    e3.set_range(duel, LOCATION_GRAVE);
    e3.set_count_limit(duel, 1, ID);
    e3.set_cost(duel, Cost::self_banish);
    e3.set_target(duel, tgtg);
    e3.set_operation(duel, tgop);
    e3.set_hint_timing(duel, 0, TIMING_STANDBY_PHASE | TIMING_MAIN_END | TIMINGS_CHECK_MONSTER_E);
    c.register_effect(duel, e3);
}

fn lptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(duel.check_lp_cost(1 - a.tp, 100));
    }
    duel.set_operation_info(0, CATEGORY_RECOVER, None, 0, a.tp, 100);
    Ok(false)
}

fn lpop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    if duel.recover(tp, 100, REASON_EFFECT)? > 0 && duel.check_lp_cost(1 - tp, 100) {
        duel.pay_lp_cost(1 - tp, 100)?;
    }
    Ok(())
}

fn costfilter(c: CardId, duel: &Duel, exc: CardId) -> bool {
    c.is_set_card(duel, &[SET_YUMMY])
        && c.is_faceup(duel)
        && c.is_able_to_grave_as_cost(duel)
        && duel.is_existing_matching_card(
            CardId::is_able_to_hand,
            0,
            LOCATION_ONFIELD | LOCATION_GRAVE,
            LOCATION_ONFIELD | LOCATION_GRAVE,
            1,
            Some(c.get_equip_group(duel).union(c.into()).union(exc.into())),
        )
}

fn retthcost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let (c, tp) = (a.e.get_handler(duel), a.tp);
    if chk == 0 {
        return Ok(c.is_status(duel, STATUS_EFFECT_ENABLED)
            && c.is_able_to_grave_as_cost(duel)
            && duel.is_existing_matching_card(|tc, duel| costfilter(tc, duel, c), tp, LOCATION_ONFIELD, 0, 1, Some(c.into())));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TOGRAVE);
    let g = duel.select_matching_card(
        tp,
        |tc, duel| costfilter(tc, duel, c),
        tp,
        LOCATION_ONFIELD,
        0,
        1,
        1,
        Some(c.into()),
    )?;
    duel.send_to_grave(g.union(c.into()), REASON_COST)?;
    Ok(false)
}

fn retthtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let locations = LOCATION_ONFIELD | LOCATION_GRAVE;
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(CardId::is_able_to_hand, a.tp, locations, locations, 1, None));
    }
    duel.set_operation_info(0, CATEGORY_TOHAND, None, 1, a.tp, locations as i32);
    Ok(false)
}

fn retthop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (tp, locations) = (a.tp, LOCATION_ONFIELD | LOCATION_GRAVE);
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_RTOHAND);
    let g = duel.select_matching_card(
        tp,
        aux::necro_valley_filter(CardId::is_able_to_hand),
        tp,
        locations,
        locations,
        1,
        1,
        None,
    )?;
    if !g.is_empty() {
        duel.hint_selection(g);
        duel.send_to_hand(g, PLAYER_NONE, REASON_EFFECT)?;
    }
    Ok(())
}

fn tgfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_YUMMY]) && c.is_able_to_grave(duel) && !c.is_code(duel, ID)
}

fn tgtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    stock::exists_target(duel, a, chk, tgfilter, LOCATION_DECK, CATEGORY_TOGRAVE)
}

fn tgop(duel: &mut Duel, a: &Args) -> Script<()> {
    stock::send_to_grave(duel, a, tgfilter, LOCATION_DECK)
}
