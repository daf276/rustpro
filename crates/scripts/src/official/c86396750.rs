// Spiritual Beast Apelio
use crate::prelude::*;
use crate::testing::{Check, GRAVE, MZONE};

const ID: u32 = 86396750;

pub const LISTED_SERIES: &[u16] = &[SET_RITUAL_BEAST];
// On the field with Ritual Beast monsters in the Graveyard to banish.
pub const SUPPORT: &[(u32, u16)] = &[(ID, MZONE), (88123329, GRAVE), (14513016, GRAVE)];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.set_sp_summon_once(duel, ID);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_ATKCHANGE);
    e1.set_type(duel, EFFECT_TYPE_QUICK_O);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_property(duel, EFFECT_FLAG_DAMAGE_STEP);
    e1.set_hint_timing(duel, TIMING_DAMAGE_STEP, TIMING_DAMAGE_STEP | TIMINGS_CHECK_MONSTER);
    e1.set_count_limit(duel, 1, 0);
    e1.set_condition(duel, aux::stat_change_damage_step_condition);
    e1.set_cost(duel, atkcost);
    e1.set_operation(duel, atkop);
    c.register_effect(duel, e1);
}

fn cfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_RITUAL_BEAST]) && c.is_able_to_remove_as_cost(duel) && aux::sp_elim_filter(c, duel, true, false)
}

fn atkcost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    stock::banish_cost(duel, a, chk, cfilter, LOCATION_MZONE | LOCATION_GRAVE)
}

fn atkop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    let e1 = duel.create_effect(a.e.get_handler(duel));
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_code(duel, EFFECT_UPDATE_ATTACK);
    e1.set_target_range(duel, LOCATION_MZONE, 0);
    e1.set_target_card(duel, |duel, _, c| c.is_set_card(duel, &[SET_RITUAL_BEAST]));
    e1.set_value(duel, 500);
    e1.set_reset(duel, RESET_PHASE | PHASE_END);
    duel.register_effect(e1, tp);
    let e2 = e1.clone_effect(duel);
    e2.set_code(duel, EFFECT_UPDATE_DEFENSE);
    duel.register_effect(e2, tp);
    Ok(())
}
