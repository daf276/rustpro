// Spright Gamma Burst
use crate::prelude::*;
use crate::testing::{Check, GRAVE, HAND, MZONE};

const ID: u32 = 42431833;

// Psychic Kappa (Level 2) and Gigantic Spright (Rank 2) to boost, and a copy in the GY to banish.
pub const SUPPORT: &[(u32, u16)] = &[(ID, HAND), (ID, GRAVE), (7892180, MZONE), (7892180, MZONE), (54498517, MZONE)];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_ATKCHANGE + CATEGORY_DEFCHANGE);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_property(duel, EFFECT_FLAG_DAMAGE_STEP);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_hint_timing(duel, TIMING_DAMAGE_STEP, TIMING_DAMAGE_STEP);
    e1.set_count_limit(duel, 1, ID);
    e1.set_condition(duel, aux::stat_change_damage_step_condition);
    e1.set_target(duel, target);
    e1.set_operation(duel, activate);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_ATKCHANGE);
    e2.set_type(duel, EFFECT_TYPE_IGNITION);
    e2.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e2.set_range(duel, LOCATION_GRAVE);
    e2.set_count_limit(duel, 1, ID);
    e2.set_cost(duel, Cost::self_banish);
    e2.set_target(duel, atktg);
    e2.set_operation(duel, atkop);
    c.register_effect(duel, e2);
}

fn atkfilter(c: CardId, duel: &Duel) -> bool {
    c.is_faceup(duel) && (c.is_level(duel, &[2]) || c.is_rank(duel, &[2]) || c.is_link(duel, &[2]))
}

fn target(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    Ok(chk == 0 && duel.is_existing_matching_card(atkfilter, a.tp, LOCATION_MZONE, LOCATION_MZONE, 1, None))
}

fn activate(duel: &mut Duel, a: &Args) -> Script<()> {
    let tg = duel.get_matching_group(atkfilter, a.tp, LOCATION_MZONE, LOCATION_MZONE, None);
    let c = a.e.get_handler(duel);
    for sc in tg {
        let e1 = duel.create_effect(c);
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_code(duel, EFFECT_UPDATE_ATTACK);
        e1.set_reset(duel, RESETS_STANDARD_PHASE_END);
        e1.set_value(duel, 1400);
        sc.register_effect(duel, e1);
        let e2 = e1.clone_effect(duel);
        e2.set_code(duel, EFFECT_UPDATE_DEFENSE);
        sc.register_effect(duel, e2);
    }
    Ok(())
}

fn atktg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_MZONE) && chkc.is_controler(duel, tp) && chkc.is_faceup(duel) && atkfilter(chkc, duel));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(atkfilter, tp, LOCATION_MZONE, 0, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_FACEUP);
    duel.select_target(tp, atkfilter, tp, LOCATION_MZONE, 0, 1, 1, None)?;
    Ok(false)
}

fn atkop(duel: &mut Duel, a: &Args) -> Script<()> {
    if let Some(tc) = duel.get_first_target()
        && tc.is_relate_to_effect(duel, a.e)
        && tc.is_faceup(duel)
    {
        let e1 = duel.create_effect(a.e.get_handler(duel));
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_code(duel, EFFECT_UPDATE_ATTACK);
        e1.set_reset(duel, RESETS_STANDARD_PHASE_END | RESET_OPPO_TURN);
        e1.set_value(duel, 1400);
        tc.register_effect(duel, e1);
    }
    Ok(())
}
