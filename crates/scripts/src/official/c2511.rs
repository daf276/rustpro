// Labrynth Cooclock
use crate::prelude::*;
use crate::testing::{Check, DECK, GRAVE, HAND, MZONE};

const ID: u32 = 2511;

pub const LISTED_NAMES: &[u32] = &[ID];
pub const LISTED_SERIES: &[u16] = &[SET_LABRYNTH];

// In the hand and Graveyard, with Labrynth Stovie Torbie in the hand to send itself and discard as a cost (Welcome Labrynth in the
// Deck to Set), and Arias the Labrynth Butler on the field.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, GRAVE),
    (74018812, HAND),
    (74018812, HAND),
    (5380979, DECK),
    (5380979, DECK),
    (73602965, MZONE),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_type(duel, EFFECT_TYPE_QUICK_O);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_range(duel, LOCATION_HAND);
    e1.set_hint_timing(duel, 0, TIMINGS_CHECK_MONSTER | TIMING_MAIN_END);
    e1.set_count_limit(duel, 1, ID);
    e1.set_cost(duel, Cost::self_discard);
    e1.set_operation(duel, acop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_TOHAND + CATEGORY_SPECIAL_SUMMON);
    e2.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e2.set_property(duel, EFFECT_FLAG_DELAY);
    e2.set_code(duel, EVENT_TO_GRAVE);
    e2.set_range(duel, LOCATION_GRAVE);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_condition(duel, thcon);
    e2.set_target(duel, thtg);
    e2.set_operation(duel, thop);
    c.register_effect(duel, e2);
}

fn acop(duel: &mut Duel, a: &Args) -> Script<()> {
    let e1 = duel.create_effect(a.e.get_handler(duel));
    e1.set_description(duel, aux::stringid(ID, 2));
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_code(duel, EFFECT_TRAP_ACT_IN_SET_TURN);
    e1.set_property(duel, EFFECT_FLAG_SET_AVAILABLE + EFFECT_FLAG_CLIENT_HINT);
    e1.set_count_limit(duel, 1, (ID, 2));
    e1.set_target_range(duel, LOCATION_SZONE, 0);
    e1.set_condition(duel, accon);
    e1.set_target_card(duel, |duel, _, c| c.is_normal_trap(duel));
    e1.set_reset(duel, RESET_PHASE | PHASE_END);
    duel.register_effect(e1, a.tp);
    Ok(())
}

fn accon(duel: &Duel, a: &Args) -> bool {
    let filter = aux::faceup_filter(|c, duel| c.is_set_card(duel, &[SET_LABRYNTH]));
    duel.is_existing_matching_card(filter, a.e.get_handler_player(duel), LOCATION_MZONE, 0, 1, None)
}

fn thcon(duel: &Duel, a: &Args) -> bool {
    let Some(re) =
        a.re.filter(|&re| a.rp == a.tp && a.r & REASON_COST == REASON_COST && re.is_activated(duel))
    else {
        return false;
    };
    let rc = re.get_handler(duel);
    a.eg.unwrap_or_default().is_exists(
        duel,
        |c, duel| c.is_previous_location(duel, LOCATION_HAND),
        1,
        Some(a.e.get_handler(duel).into()),
    ) && ((re.is_trap_effect(duel) && re.is_has_type(duel, EFFECT_TYPE_ACTIVATE))
        || (rc.is_set_card(duel, &[SET_LABRYNTH]) && !rc.is_code(duel, ID)))
}

fn thtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if chk == 0 {
        return Ok(c.is_able_to_hand(duel)
            || (duel.get_location_count(tp, LOCATION_MZONE) > 0 && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)));
    }
    duel.set_operation_info(0, CATEGORY_LEAVE_GRAVE, Some(c.into()), 1, tp, 0);
    duel.set_possible_operation_info(0, CATEGORY_TOHAND, Some(c.into()), 1, tp, 0);
    duel.set_possible_operation_info(0, CATEGORY_SPECIAL_SUMMON, Some(c.into()), 1, tp, 0);
    Ok(false)
}

fn thop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if !c.is_relate_to_effect(duel, e) {
        return Ok(());
    }
    aux::to_hand_or_else(
        duel,
        c,
        tp,
        |sc, duel| sc.is_can_be_special_summoned(duel, e, 0, tp, false, false) && duel.get_location_count(tp, LOCATION_MZONE) > 0,
        |duel| duel.special_summon(c, 0, tp, tp, false, false, POS_FACEUP).map(drop),
        aux::stringid(ID, 3),
    )
}
