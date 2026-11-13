// Quick Summon
use crate::prelude::*;
use crate::testing::{Check, DECK, GRAVE, HAND, MZONE, OPPONENT};

const ID: u32 = 10086952;

// In the hand and the Graveyard, with Level 6 Sky Dragons in the hand and Deck to Summon, and Sangan on the field of each player to
// Tribute and to allow the Summon without Tributing.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, GRAVE),
    (ID, GRAVE),
    (95288024, HAND),
    (95288024, HAND),
    (95288024, DECK),
    (95288024, DECK),
    (26202165, MZONE),
    (26202165, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_SUMMON);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_target(duel, target);
    e1.set_operation(duel, activate);
    e1.set_hint_timing(duel, 0, TIMING_STANDBY_PHASE | TIMING_MAIN_END | TIMINGS_CHECK_MONSTER_E);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_TOHAND + CATEGORY_SEARCH + CATEGORY_SUMMON);
    e2.set_type(duel, EFFECT_TYPE_IGNITION);
    e2.set_range(duel, LOCATION_GRAVE);
    e2.set_count_limit(duel, 1, ID);
    e2.set_condition(duel, aux::exccon);
    e2.set_cost(duel, Cost::self_banish);
    e2.set_target(duel, thtg);
    e2.set_operation(duel, thop);
    c.register_effect(duel, e2);
}

fn no_tribute_summon_effect(duel: &mut Duel, handler: CardId) -> EffectId {
    let e1 = duel.create_effect(handler);
    e1.set_description(duel, aux::stringid(ID, 2));
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_code(duel, EFFECT_SUMMON_PROC);
    e1.set_condition(duel, |_, a| a.c.is_none() || a.min == 0);
    e1
}

fn nsfilter(c: CardId, duel: &Duel, no_tribute_chk: bool, handler: CardId) -> bool {
    if c.can_summon_or_set(duel, true, None, 0) {
        return true;
    }
    if c.is_level_above(duel, 5) && c.is_location(duel, LOCATION_HAND) && no_tribute_chk {
        return duel.check_mut(|duel| {
            let e1 = no_tribute_summon_effect(duel, handler);
            c.register_effect(duel, e1);
            let res = c.is_summonable(duel, true, None);
            e1.reset(duel);
            res
        });
    }
    false
}

fn no_tribute_check(duel: &Duel, tp: u8) -> bool {
    duel.get_field_group_count(tp, 0, LOCATION_MZONE) > 0 && duel.get_location_count(tp, LOCATION_MZONE) > 0
}

fn target(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        let (no_tribute_chk, handler) = (no_tribute_check(duel, tp), a.e.get_handler(duel));
        return Ok(duel.is_existing_matching_card(
            |c, duel| nsfilter(c, duel, no_tribute_chk, handler),
            tp,
            LOCATION_HAND | LOCATION_MZONE,
            0,
            1,
            None,
        ));
    }
    duel.set_operation_info(0, CATEGORY_SUMMON, None, 1, tp, (LOCATION_HAND | LOCATION_MZONE) as i32);
    Ok(false)
}

fn activate(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    let no_tribute_chk = no_tribute_check(duel, tp);
    let c = a.e.get_handler(duel);
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SUMMON);
    let g = duel.select_matching_card(
        tp,
        |sc, duel| nsfilter(sc, duel, no_tribute_chk, c),
        tp,
        LOCATION_HAND | LOCATION_MZONE,
        0,
        1,
        1,
        None,
    )?;
    let Some(sc) = g.get_first() else { return Ok(()) };
    if sc.is_level_above(duel, 5) && sc.is_location(duel, LOCATION_HAND) && no_tribute_chk {
        let e1 = no_tribute_summon_effect(duel, c);
        e1.set_reset(duel, RESET_EVENT | RESETS_STANDARD);
        sc.register_effect(duel, e1);
    }
    duel.summon_or_set(tp, sc, true, None, 0)
}

fn thfilter(c: CardId, duel: &Duel, tp: u8) -> bool {
    let (min_tributes, max_tributes) = c.get_tribute_requirement(duel);
    c.is_level_above(duel, 5)
        && c.is_summonable_card(duel)
        && duel.check_tribute(c, min_tributes, max_tributes, None, c.get_controler(duel), 0x1f)
        && c.can_summon_or_set(duel, true, None, 1)
        && duel.is_player_can_summon_full(tp, SUMMON_TYPE_TRIBUTE, c)
        && c.is_able_to_hand(duel)
}

fn thtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(|c, duel| thfilter(c, duel, tp), tp, LOCATION_DECK | LOCATION_GRAVE, 0, 1, None));
    }
    duel.set_operation_info(0, CATEGORY_TOHAND, None, 1, tp, (LOCATION_DECK | LOCATION_GRAVE) as i32);
    duel.set_operation_info(0, CATEGORY_SUMMON, None, 1, tp, LOCATION_HAND as i32);
    Ok(false)
}

fn thop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_ATOHAND);
    let filter = aux::necro_valley_filter(|c, duel| thfilter(c, duel, tp));
    let g = duel.select_matching_card(tp, filter, tp, LOCATION_DECK | LOCATION_GRAVE, 0, 1, 1, None)?;
    let Some(sc) = g.get_first() else { return Ok(()) };
    if sc.is_location(duel, LOCATION_GRAVE) {
        duel.hint_selection(sc);
    }
    if duel.send_to_hand(sc, PLAYER_NONE, REASON_EFFECT)? > 0 {
        if sc.is_previous_location(duel, LOCATION_DECK) {
            duel.confirm_cards(1 - tp, sc)?;
        }
        duel.shuffle_hand(tp)?;
        duel.break_effect()?;
        duel.summon_or_set(tp, sc, true, None, 1)?;
    }
    Ok(())
}
