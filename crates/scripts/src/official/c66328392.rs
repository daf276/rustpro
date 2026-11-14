// Deception of the Sinful Spoils
use crate::prelude::*;
use crate::testing::{Check, DECK, HAND, MZONE, OPPONENT};

const ID: u32 = 66328392;

pub const LISTED_SERIES: &[u16] = &[SET_AZAMINA];
// In the hand with Elzette (an Azamina monster) in the Deck and on the field and Raigeki and Mystical Space Typhoon in the hand,
// against monsters on the opponent's field.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (38648860, DECK),
    (38648860, MZONE),
    (12580477, HAND),
    (5318639, HAND),
    (5318639, HAND),
    (13597785, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
    Check::ResolvesEffect(aux::stringid(ID, 2)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e0 = duel.create_effect(c);
    e0.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e0.set_code(duel, EVENT_FREE_CHAIN);
    c.register_effect(duel, e0);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_TOHAND + CATEGORY_SEARCH);
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_range(duel, LOCATION_SZONE);
    e1.set_count_limit(duel, 1, ID);
    e1.set_cost(duel, thcost);
    e1.set_target(duel, thtg);
    e1.set_operation(duel, thop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_RECOVER);
    e2.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e2.set_property(duel, EFFECT_FLAG_DELAY);
    e2.set_code(duel, EVENT_TO_GRAVE);
    e2.set_range(duel, LOCATION_SZONE);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_condition(duel, lpcon);
    e2.set_target(duel, lptg);
    e2.set_operation(duel, lpop);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 2));
    e3.set_category(duel, CATEGORY_LEAVE_GRAVE + CATEGORY_SET);
    e3.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e3.set_code(duel, EVENT_PHASE + PHASE_END);
    e3.set_range(duel, LOCATION_GRAVE);
    e3.set_count_limit(duel, 1, (ID, 2));
    e3.set_condition(duel, setcon);
    e3.set_target(duel, settg);
    e3.set_operation(duel, setop);
    c.register_effect(duel, e3);
}

fn thcost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.check_release_group_cost(tp, aux::TRUE, (1, 1), true, None, None));
    }
    let g = duel.select_release_group_cost(tp, aux::TRUE, (1, 1), true, None, None)?;
    duel.release(g, REASON_COST)?;
    Ok(false)
}

fn thfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_AZAMINA]) && c.is_able_to_hand(duel)
}

fn thtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    stock::exists_target(duel, a, chk, thfilter, LOCATION_DECK, CATEGORY_TOHAND)
}

fn thop(duel: &mut Duel, a: &Args) -> Script<()> {
    stock::search(duel, a, thfilter, LOCATION_DECK)
}

fn lpcon(duel: &Duel, a: &Args) -> bool {
    let tp = a.tp;
    a.eg.unwrap_or_default()
        .is_exists(duel, |c, duel| c.is_controler(duel, 1 - tp) && c.is_monster(duel), 1, None)
}

fn lptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        let filter = aux::faceup_filter(|c, duel| c.is_set_card(duel, &[SET_AZAMINA]));
        return Ok(duel.is_existing_matching_card(filter, tp, LOCATION_MZONE, 0, 1, None));
    }
    duel.set_operation_info(0, CATEGORY_RECOVER, None, 0, tp, 1500);
    Ok(false)
}

fn lpop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    let lp = duel.get_lp(1 - tp);
    duel.set_lp(1 - tp, lp - 1500);
    duel.recover(tp, 1500, REASON_EFFECT)?;
    Ok(())
}

fn setcon(duel: &Duel, a: &Args) -> bool {
    let c = a.e.get_handler(duel);
    c.get_turn_id(duel) == duel.get_turn_count()
        && c.is_previous_location(duel, LOCATION_STZONE)
        && c.is_previous_position(duel, POS_FACEUP)
}

fn settg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let c = a.e.get_handler(duel);
    if chk == 0 {
        return Ok(c.is_ssetable(duel));
    }
    duel.set_operation_info(0, CATEGORY_LEAVE_GRAVE, Some(c.into()), 1, a.tp, 0);
    Ok(false)
}

fn setop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if c.is_relate_to_effect(duel, e) && c.is_ssetable(duel) {
        duel.sset(tp, c)?;
    }
    Ok(())
}
