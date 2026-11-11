// Windwitch - Blizzard Bell
use crate::prelude::*;
use crate::testing::{Check, HAND, MZONE, OPPONENT};

const ID: u32 = 84851250;

pub const LISTED_NAMES: &[u32] = &[ID];

// Windwitch - Glass Bell on the field for the condition, Windwitch - Snow Bell and a copy in the hand, and Raigeki in the opponent's
// hand to give the opponent a Main Phase.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, HAND),
    (71007216, MZONE),
    (70117860, HAND),
    (12580477, HAND | OPPONENT),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 1)), Check::TributeSummoned(0)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE + EFFECT_FLAG_UNCOPYABLE);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_code(duel, EFFECT_SUMMON_PROC);
    e1.set_condition(duel, ntcon);
    e1.set_value(duel, 1);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_category(duel, CATEGORY_DAMAGE);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_type(duel, EFFECT_TYPE_QUICK_O);
    e2.set_code(duel, EVENT_FREE_CHAIN);
    e2.set_hint_timing(duel, 0, TIMINGS_CHECK_MONSTER | TIMING_MAIN_END);
    e2.set_range(duel, LOCATION_HAND | LOCATION_MZONE);
    e2.set_cost(duel, Cost::self_to_grave);
    e2.set_condition(duel, condition);
    e2.set_target(duel, target);
    e2.set_operation(duel, stock::damage);
    e2.set_count_limit(duel, 1, ID);
    c.register_effect(duel, e2);
}

fn filter(c: CardId, duel: &Duel) -> bool {
    c.is_facedown(duel) || !c.is_set_card(duel, &[SET_WINDWITCH])
}

fn ntcon(duel: &Duel, a: &Args) -> bool {
    let Some(c) = a.c else { return true };
    let tp = c.get_controler(duel);
    a.min == 0
        && c.get_level(duel) > 4
        && duel.get_location_count_full(tp, LOCATION_MZONE, tp, LOCATION_REASON_TOFIELD, a.zone) > 0
        && (duel.get_field_group_count(tp, LOCATION_MZONE, 0) == 0
            || !duel.is_existing_matching_card(filter, tp, LOCATION_MZONE, 0, 1, None))
}

fn cfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_WINDWITCH]) && c.is_faceup(duel) && !c.is_code(duel, ID)
}

fn condition(duel: &Duel, a: &Args) -> bool {
    let tp = a.tp;
    duel.is_existing_matching_card(cfilter, tp, LOCATION_MZONE, 0, 1, None) && duel.is_turn_player(1 - tp) && duel.is_main_phase()
}

fn target(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(true);
    }
    let tp = a.tp;
    duel.set_target_player(1 - tp);
    duel.set_target_param(500);
    duel.set_operation_info(0, CATEGORY_DAMAGE, None, 0, 1 - tp, 500);
    Ok(false)
}
