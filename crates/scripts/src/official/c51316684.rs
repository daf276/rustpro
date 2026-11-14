// Infernoid Seitsemas
use crate::prelude::*;
use crate::testing::{Check, GRAVE, HAND, MZONE, OPPONENT};

use super::c60551528::{rmfilter, rmop, rmtg};

const ID: u32 = 51316684;

pub const LISTED_SERIES: &[u16] = &[SET_INFERNOID];

// In the hand to Special Summon by banishing Infernoid Decatron and Pirmais from the GY, with Sangan to battle and Tribute.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, MZONE),
    (ID, MZONE),
    (58446973, GRAVE),
    (58446973, GRAVE),
    (60551528, GRAVE),
    (60551528, GRAVE),
    (26202165, MZONE),
    (26202165, MZONE | OPPONENT),
    (26202165, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Infernoid::register_summon_procedure::<2>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_REMOVE);
    e1.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e1.set_code(duel, EVENT_PHASE | PHASE_BATTLE);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_count_limit(duel, 1, 0);
    e1.set_condition(duel, rmcon);
    e1.set_target(duel, rmtg1);
    e1.set_operation(duel, rmop1);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_REMOVE);
    e2.set_type(duel, EFFECT_TYPE_QUICK_O);
    e2.set_code(duel, EVENT_FREE_CHAIN);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e2.set_count_limit(duel, 1, 0);
    e2.set_cost(duel, rmcost2);
    e2.set_target(duel, rmtg);
    e2.set_operation(duel, rmop);
    c.register_effect(duel, e2);
}

fn rmcon(duel: &Duel, a: &Args) -> bool {
    duel.is_turn_player(a.tp) && !a.e.get_handler(duel).get_battled_group(duel).is_empty()
}

fn rmtg1(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(CardId::is_able_to_remove, tp, LOCATION_ONFIELD, LOCATION_ONFIELD, 1, None));
    }
    let g = duel.get_matching_group(CardId::is_able_to_remove, tp, LOCATION_ONFIELD, LOCATION_ONFIELD, None);
    duel.set_operation_info(0, CATEGORY_REMOVE, Some(g), 1, 0, 0);
    Ok(false)
}

fn rmop1(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_REMOVE);
    let g = duel.select_matching_card(tp, CardId::is_able_to_remove, tp, LOCATION_ONFIELD, LOCATION_ONFIELD, 1, 1, None)?;
    if !g.is_empty() {
        duel.remove(g, POS_FACEUP, REASON_EFFECT)?;
    }
    Ok(())
}

fn rmcost2(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let dg = duel.get_matching_group(|c, duel| rmfilter(c, duel, Some(e)), tp, 0, LOCATION_MZONE | LOCATION_GRAVE, None);
    let check = aux::release_check_target(dg);
    if chk == 0 {
        return Ok(duel.check_release_group_cost(tp, CardId::is_releasable, (1, 1), false, Some(&check), None));
    }
    let g = duel.select_release_group_cost(tp, CardId::is_releasable, (1, 1), false, Some(&check), None)?;
    duel.release(g, REASON_COST)?;
    Ok(false)
}
