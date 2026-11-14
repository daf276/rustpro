// Dinomorphia Brute
use crate::prelude::*;
use crate::testing::{Check, GRAVE, HAND, SZONE};

const ID: u32 = 99414629;

pub const LISTED_SERIES: &[u16] = &[SET_DINOMORPHIA];
// Set, with Dinomorphia Shell to summon a Dinomorphia Token on the opponent's Battle Phase, and a copy in the GY.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, SZONE),
    (ID, SZONE),
    (ID, GRAVE),
    (25419323, SZONE),
    (25419323, SZONE),
    (25419323, HAND),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_DESTROY);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_count_limit_full(duel, 1, ID, EFFECT_COUNT_CODE_OATH);
    e1.set_hint_timing(duel, 0, TIMINGS_CHECK_MONSTER_E);
    e1.set_cost(duel, super::c7336745::cost);
    e1.set_target(duel, target);
    e1.set_operation(duel, activate);
    c.register_effect(duel, e1);
    register_no_effect_damage(duel, c, ID);
}

// The Graveyard effect of Alert and Brute: banish this card so its controller takes no effect damage from the opponent.
pub fn register_no_effect_damage(duel: &mut Duel, c: CardId, id: u32) {
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(id, 1));
    e2.set_type(duel, EFFECT_TYPE_QUICK_O);
    e2.set_code(duel, EVENT_CHAINING);
    e2.set_range(duel, LOCATION_GRAVE);
    e2.set_condition(duel, |duel, a| a.rp == 1 - a.tp && duel.get_lp(a.tp) <= 2000);
    e2.set_cost(duel, Cost::self_banish);
    e2.set_operation(duel, nodamop);
    c.register_effect(duel, e2);
}

fn nodamop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    let e1 = duel.create_effect(a.e.get_handler(duel));
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_code(duel, EFFECT_CHANGE_DAMAGE);
    e1.set_property(duel, EFFECT_FLAG_PLAYER_TARGET);
    e1.set_target_range(duel, 1, 0);
    e1.set_value(duel, Value::ChangeDamage(damval));
    e1.set_reset(duel, RESET_PHASE | PHASE_END);
    duel.register_effect(e1, tp);
    let e2 = e1.clone_effect(duel);
    e2.set_code(duel, EFFECT_NO_EFFECT_DAMAGE);
    e2.set_reset(duel, RESET_PHASE | PHASE_END);
    duel.register_effect(e2, tp);
    Ok(())
}

fn damval(duel: &mut Duel, e: EffectId, _: Option<EffectId>, val: u32, r: u32, rp: u8, _: Option<CardId>) -> u32 {
    if r & REASON_EFFECT != 0 && rp != e.get_owner_player(duel) {
        0
    } else {
        val
    }
}

fn desfilter(c: CardId, duel: &Duel, tp: u8) -> bool {
    (c.is_faceup(duel) && c.is_set_card(duel, &[SET_DINOMORPHIA]) && c.is_monster(duel) && c.is_controler(duel, tp))
        || c.is_controler(duel, 1 - tp)
}

fn destroy_group(duel: &Duel, tp: u8) -> CardSet {
    duel.get_matching_group(|c, duel| desfilter(c, duel, tp), tp, LOCATION_ONFIELD, LOCATION_ONFIELD, None)
}

fn rescon(tp: u8) -> impl Fn(&Duel, CardSet, CardSet, Option<CardId>) -> (bool, bool) {
    move |duel, sg, _, _| (sg.filter_count(duel, |c, duel| c.is_controler(duel, tp), None) == 1, false)
}

fn target(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let rg = destroy_group(duel, a.tp);
    if chk == 0 {
        return Ok(aux::select_unselect_group_check(duel, rg, 2, 2, &rescon(a.tp)));
    }
    duel.set_operation_info(0, CATEGORY_DESTROY, Some(rg), 2, 0, 0);
    Ok(false)
}

fn activate(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    let rg = destroy_group(duel, tp);
    let g = aux::select_unselect_group(duel, rg, 2, 2, &rescon(tp), tp, HINTMSG_DESTROY)?;
    if g.len() == 2 {
        duel.hint_selection_full(g, true);
        duel.destroy(g, REASON_EFFECT)?;
    }
    Ok(())
}
