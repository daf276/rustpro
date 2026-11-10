// Limiter Removal
use crate::prelude::*;
use crate::testing::{HAND, MZONE};

const ID: u32 = 23171610;

// Cyber Dragon, a Machine, on the field to double the ATK of.
pub const SUPPORT: &[(u32, u16)] = &[(ID, HAND), (CARD_CYBER_DRAGON, MZONE)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_category(duel, CATEGORY_ATKCHANGE);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_property(duel, EFFECT_FLAG_DAMAGE_STEP);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_hint_timing(duel, TIMING_DAMAGE_STEP, TIMING_DAMAGE_STEP);
    e1.set_condition(duel, aux::stat_change_damage_step_condition);
    e1.set_target(duel, target);
    e1.set_operation(duel, activate);
    c.register_effect(duel, e1);
}

fn target(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        let filter = aux::faceup_filter(|c, duel| c.is_race(duel, RACE_MACHINE));
        return Ok(duel.is_existing_matching_card(filter, a.tp, LOCATION_MZONE, 0, 1, None));
    }
    Ok(false)
}

fn filter2(c: CardId, duel: &Duel, e: EffectId) -> bool {
    c.is_faceup(duel) && c.is_race(duel, RACE_MACHINE) && !c.is_immune_to_effect(duel, e)
}

fn activate(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let sg = duel.get_matching_group(|c, duel| filter2(c, duel, e), tp, LOCATION_MZONE, 0, None);
    let c = e.get_handler(duel);
    let fid = c.get_field_id(duel);
    for tc in sg {
        let e1 = duel.create_effect(c);
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_code(duel, EFFECT_SET_ATTACK_FINAL);
        e1.set_reset(duel, RESETS_STANDARD_PHASE_END);
        e1.set_value(duel, tc.get_attack(duel) * 2);
        tc.register_effect(duel, e1);
        tc.register_flag_effect_full(duel, ID, RESETS_STANDARD_PHASE_END, 0, 1, fid.into(), 0);
    }
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_CONTINUOUS);
    e2.set_property(duel, EFFECT_FLAG_IGNORE_IMMUNE);
    e2.set_code(duel, EVENT_PHASE + PHASE_END);
    e2.set_reset(duel, RESET_PHASE | PHASE_END);
    e2.set_count_limit(duel, 1, 0);
    e2.set_label(duel, fid.into());
    e2.set_label_object_group(duel, Some(sg));
    e2.set_condition_mut(duel, descon);
    e2.set_operation(duel, desop);
    duel.register_effect(e2, tp);
    Ok(())
}

fn desfilter(c: CardId, duel: &Duel, fid: i64) -> bool {
    c.get_flag_effect_label(duel, ID) == Some(fid)
}

fn descon(duel: &mut Duel, a: &Args) -> bool {
    let g = a.e.get_label_object_group(duel).unwrap_or_default();
    let fid = a.e.get_label(duel);
    if !g.is_exists(duel, |c, duel| desfilter(c, duel, fid), 1, None) {
        a.e.reset(duel);
        return false;
    }
    true
}

fn desop(duel: &mut Duel, a: &Args) -> Script<()> {
    let g = a.e.get_label_object_group(duel).unwrap_or_default();
    let fid = a.e.get_label(duel);
    let dg = g.filter(duel, |c, duel| desfilter(c, duel, fid), None);
    duel.destroy(dg, REASON_EFFECT)?;
    Ok(())
}
