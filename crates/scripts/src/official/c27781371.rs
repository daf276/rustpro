// Regenesis Birth
use crate::prelude::*;
use crate::testing::{Check, DECK, GRAVE, HAND, MZONE, OPPONENT, REMOVED, SZONE};

const ID: u32 = 27781371;

pub const LISTED_SERIES: &[u16] = &[SET_REGENESIS];

struct SendToGrave;

impl aux::DelayedOperation for SendToGrave {
    fn operate(duel: &mut Duel, dg: CardSet, _: &Args) -> Script<()> {
        duel.send_to_grave(dg, REASON_EFFECT)?;
        Ok(())
    }
}

// Set, with Regenesis monsters in the Deck to send and in the hand, Graveyard and banishment to summon, and the opponent's
// face-up monsters to set.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, SZONE),
    (ID, SZONE),
    (22938501, DECK),
    (96540807, DECK),
    (59323650, GRAVE),
    (22938501, REMOVED),
    (96540807, HAND),
    (13597785, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e0 = duel.create_effect(c);
    e0.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e0.set_code(duel, EVENT_FREE_CHAIN);
    c.register_effect(duel, e0);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_POSITION + CATEGORY_SET);
    e1.set_type(duel, EFFECT_TYPE_QUICK_O);
    e1.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_range(duel, LOCATION_SZONE);
    e1.set_hint_timing(duel, 0, TIMING_STANDBY_PHASE | TIMING_MAIN_END | TIMINGS_CHECK_MONSTER_E);
    e1.set_count_limit(duel, 1, ID);
    e1.set_cost(duel, poscost);
    e1.set_target(duel, postg);
    e1.set_operation(duel, posop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_SPECIAL_SUMMON + CATEGORY_TOGRAVE);
    e2.set_type(duel, EFFECT_TYPE_QUICK_O);
    e2.set_code(duel, EVENT_FREE_CHAIN);
    e2.set_range(duel, LOCATION_SZONE);
    e2.set_hint_timing(duel, 0, TIMING_STANDBY_PHASE | TIMING_MAIN_END | TIMINGS_CHECK_MONSTER_E);
    e2.set_count_limit(duel, 1, ID);
    e2.set_condition(duel, |duel, a| duel.is_turn_player(1 - a.tp));
    e2.set_cost(duel, spcost);
    e2.set_target(duel, sptg);
    e2.set_operation(duel, spop);
    c.register_effect(duel, e2);
}

fn poscostfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_REGENESIS]) && c.is_monster(duel) && c.is_able_to_grave_as_cost(duel)
}

fn poscost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    stock::send_to_grave_cost(duel, a, chk, poscostfilter, LOCATION_DECK)
}

fn postg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_MZONE) && chkc.is_controler(duel, 1 - tp) && chkc.is_can_turn_set(duel));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(CardId::is_can_turn_set, tp, 0, LOCATION_MZONE, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_POSCHANGE);
    let g = duel.select_target(tp, CardId::is_can_turn_set, tp, 0, LOCATION_MZONE, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_POSITION, g, 1, tp, POS_FACEDOWN_DEFENSE as i32);
    Ok(false)
}

fn posop(duel: &mut Duel, a: &Args) -> Script<()> {
    if let Some(tc) = duel.get_first_target()
        && tc.is_relate_to_effect(duel, a.e)
    {
        duel.change_position(tc, POS_FACEDOWN_DEFENSE)?;
    }
    Ok(())
}

fn spcost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let c = a.e.get_handler(duel);
    if chk == 0 {
        return Ok(c.is_able_to_grave_as_cost(duel) && c.is_status(duel, STATUS_EFFECT_ENABLED));
    }
    duel.send_to_grave(c, REASON_COST)?;
    Ok(false)
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_set_card(duel, &[SET_REGENESIS])
        && (c.is_faceup(duel) || c.is_location(duel, LOCATION_HAND))
        && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let locations = LOCATION_HAND | LOCATION_GRAVE | LOCATION_REMOVED;
    if chk == 0 {
        return Ok(duel.get_location_count(tp, LOCATION_MZONE) > 0
            && duel.is_existing_matching_card(|c, duel| spfilter(c, duel, e, tp), tp, locations, 0, 1, None));
    }
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, 0, locations as i32);
    duel.set_possible_operation_info(0, CATEGORY_TOGRAVE, None, 1, 0, LOCATION_MZONE as i32);
    Ok(false)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let mut ft = duel.get_location_count(tp, LOCATION_MZONE);
    let filter = aux::necro_valley_filter(|c, duel| spfilter(c, duel, e, tp));
    let tg = duel.get_matching_group(filter, tp, LOCATION_HAND | LOCATION_GRAVE | LOCATION_REMOVED, 0, None);
    if tg.is_empty() || ft <= 0 {
        return Ok(());
    }
    if duel.is_player_affected_by_effect(tp, CARD_BLUEEYES_SPIRIT).is_some() {
        ft = 1;
    }
    let ct = (ft as usize).min(tg.get_class_count(duel, |c, duel| i64::from(c.get_code(duel))));
    let g = aux::select_unselect_group(duel, tg, ct, ct, &aux::dncheck, tp, HINTMSG_SPSUMMON)?;
    if !g.is_empty() && duel.special_summon(g, 0, tp, tp, false, false, POS_FACEUP)? > 0 {
        aux::delayed_operation_full::<SendToGrave>(duel, g, PHASE_END, 0, ID, e, tp, aux::stringid(ID, 2), 1);
    }
    Ok(())
}
