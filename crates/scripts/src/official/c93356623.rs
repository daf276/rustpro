// Magical Musket - Cross-Domination
use crate::prelude::*;
use crate::testing::{HAND, MZONE};

const ID: u32 = 93356623;

pub const LISTED_SERIES: &[u16] = &[SET_MAGICAL_MUSKET];
// In the hand to activate with Magical Musketeer Kidbrave on the field.
pub const SUPPORT: &[(u32, u16)] = &[(ID, HAND), (ID, HAND), (5230799, MZONE), (5230799, MZONE)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_category(duel, CATEGORY_ATKCHANGE + CATEGORY_DEFCHANGE + CATEGORY_DISABLE);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_property(duel, EFFECT_FLAG_CARD_TARGET + EFFECT_FLAG_DAMAGE_STEP);
    e1.set_count_limit_full(duel, 1, ID, EFFECT_COUNT_CODE_OATH);
    e1.set_hint_timing(duel, TIMING_DAMAGE_STEP, TIMING_DAMAGE_STEP | TIMINGS_CHECK_MONSTER);
    e1.set_condition(duel, condition);
    e1.set_target(duel, target);
    e1.set_operation(duel, activate);
    c.register_effect(duel, e1);
}

fn condition(duel: &Duel, a: &Args) -> bool {
    let musket = aux::faceup_filter(|c, duel| c.is_set_card(duel, &[SET_MAGICAL_MUSKET]));
    duel.is_existing_matching_card(musket, a.tp, LOCATION_MZONE, 0, 1, None) && aux::stat_change_damage_step_condition(duel, a)
}

fn filter(c: CardId, duel: &Duel) -> bool {
    c.is_faceup(duel) && !(c.get_attack(duel) == 0 && c.get_defense(duel) == 0 && c.is_disabled(duel))
}

fn target(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_MZONE) && filter(chkc, duel));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(filter, tp, LOCATION_MZONE, LOCATION_MZONE, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TARGET);
    duel.select_target(tp, filter, tp, LOCATION_MZONE, LOCATION_MZONE, 1, 1, None)?;
    Ok(false)
}

fn activate(duel: &mut Duel, a: &Args) -> Script<()> {
    let c = a.e.get_handler(duel);
    let Some(tc) = duel.get_first_target() else { return Ok(()) };
    if !(tc.is_faceup(duel) && tc.is_relate_to_effect(duel, a.e)) {
        return Ok(());
    }
    let e0 = duel.create_effect(c);
    e0.set_type(duel, EFFECT_TYPE_SINGLE);
    e0.set_code(duel, EFFECT_SET_ATTACK_FINAL);
    e0.set_value(duel, 0);
    e0.set_reset(duel, RESETS_STANDARD_PHASE_END);
    tc.register_effect(duel, e0);
    let e1 = e0.clone_effect(duel);
    e1.set_code(duel, EFFECT_SET_DEFENSE_FINAL);
    tc.register_effect(duel, e1);
    duel.negate_related_chain(tc, RESET_TURN_SET);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_SINGLE);
    e2.set_code(duel, EFFECT_DISABLE);
    e2.set_reset(duel, RESETS_STANDARD_PHASE_END);
    tc.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_type(duel, EFFECT_TYPE_SINGLE);
    e3.set_code(duel, EFFECT_DISABLE_EFFECT);
    e3.set_value(duel, RESET_TURN_SET as i32);
    e3.set_reset(duel, RESETS_STANDARD_PHASE_END);
    tc.register_effect(duel, e3);
    Ok(())
}
