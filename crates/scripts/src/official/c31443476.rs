// Quick Launch
use crate::prelude::*;
use crate::testing::{Check, DECK, HAND};

const ID: u32 = 31443476;

// In the hand with Rokket Tracer in the Deck to Special Summon.
pub const SUPPORT: &[(u32, u16)] = &[(ID, HAND), (68464358, DECK), (68464358, DECK)];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(0), Check::SpecialSummonedCode(68464358)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_target(duel, target);
    e1.set_operation(duel, activate);
    e1.set_hint_timing(duel, 0, TIMING_END_PHASE);
    c.register_effect(duel, e1);
}

fn filter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_set_card(duel, &[SET_ROKKET]) && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
}

fn target(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    stock::special_summon_target(duel, a, chk, |c, duel| filter(c, duel, a.e, a.tp), LOCATION_DECK)
}

fn activate(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    if duel.get_location_count(tp, LOCATION_MZONE) <= 0 {
        return Ok(());
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
    let g = duel.select_matching_card(tp, |c, duel| filter(c, duel, e, tp), tp, LOCATION_DECK, 0, 1, 1, None)?;
    if let Some(tc) = g.get_first()
        && duel.special_summon_step(tc, 0, tp, tp, false, false, POS_FACEUP)?
    {
        let handler = e.get_handler(duel);
        let e1 = duel.create_effect(handler);
        e1.set_description(duel, 3206);
        e1.set_property(duel, EFFECT_FLAG_CLIENT_HINT);
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_code(duel, EFFECT_CANNOT_ATTACK);
        e1.set_reset(duel, RESET_EVENT | RESETS_STANDARD);
        tc.register_effect_full(duel, e1, true);
        tc.register_flag_effect(duel, ID, RESET_EVENT | RESETS_STANDARD, 0, 1);
        let e2 = duel.create_effect(handler);
        e2.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_CONTINUOUS);
        e2.set_code(duel, EVENT_PHASE + PHASE_END);
        e2.set_property(duel, EFFECT_FLAG_IGNORE_IMMUNE);
        e2.set_label_object_group(duel, Some(tc.into()));
        e2.set_condition_mut(duel, descon);
        e2.set_operation(duel, desop);
        e2.set_reset(duel, RESET_PHASE | PHASE_END);
        e2.set_count_limit(duel, 1, 0);
        duel.register_effect(e2, tp);
    }
    duel.special_summon_complete()?;
    Ok(())
}

fn descon(duel: &mut Duel, a: &Args) -> bool {
    let tc = a.e.get_label_object_group(duel).and_then(|g| g.get_first());
    if tc.is_some_and(|tc| tc.get_flag_effect(duel, ID) != 0) {
        return true;
    }
    a.e.reset(duel);
    false
}

fn desop(duel: &mut Duel, a: &Args) -> Script<()> {
    duel.destroy(a.e.get_label_object_group(duel).unwrap_or_default(), REASON_EFFECT)?;
    Ok(())
}
