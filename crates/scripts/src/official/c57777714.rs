// Ritual Beast Tamer Zeframpilica
use crate::prelude::*;
use crate::testing::{Check, GRAVE, HAND};

const ID: u32 = 57777714;

pub const LISTED_SERIES: &[u16] = &[SET_RITUAL_BEAST, SET_ZEFRA];
pub const LISTED_NAMES: &[u32] = &[ID];
// Copies in the hand to Normal Summon, with Ritual Beasts in the Graveyard to Special Summon.
pub const SUPPORT: &[(u32, u16)] = &[(ID, HAND), (ID, HAND), (ID, HAND), (14513016, GRAVE), (88123329, GRAVE)];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(0)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.set_sp_summon_once(duel, ID);
    Pendulum::add_procedure(duel, c);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_FIELD);
    e2.set_range(duel, LOCATION_PZONE);
    e2.set_code(duel, EFFECT_CANNOT_SPECIAL_SUMMON);
    e2.set_property(
        duel,
        EFFECT_FLAG_PLAYER_TARGET + EFFECT_FLAG_CANNOT_DISABLE + EFFECT_FLAG_CANNOT_NEGATE,
    );
    e2.set_target_range(duel, 1, 0);
    e2.set_target_spsummon(duel, splimit);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e3.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e3.set_code(duel, EVENT_SUMMON_SUCCESS);
    e3.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e3.set_target(duel, target);
    e3.set_operation(duel, operation);
    c.register_effect(duel, e3);
    let e4 = e3.clone_effect(duel);
    e4.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e4.set_condition(duel, |duel, a| a.e.get_handler(duel).is_pendulum_summoned(duel));
    c.register_effect(duel, e4);
}

fn splimit(duel: &Duel, _: EffectId, c: CardId, _: u8, sumtype: u32, _: u8, _: u8, _: Option<EffectId>) -> bool {
    if c.is_set_card(duel, &[SET_RITUAL_BEAST]) || c.is_set_card(duel, &[SET_ZEFRA]) {
        return false;
    }
    sumtype & SUMMON_TYPE_PENDULUM == SUMMON_TYPE_PENDULUM
}

fn filter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    (c.is_set_card(duel, &[SET_RITUAL_BEAST]) || c.is_set_card(duel, &[SET_ZEFRA]))
        && !c.is_code(duel, ID)
        && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
}

fn target(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_GRAVE) && chkc.is_controler(duel, tp) && filter(chkc, duel, e, tp));
    }
    let f = |c, duel: &Duel| filter(c, duel, e, tp);
    if chk == 0 {
        return Ok(duel.get_location_count(tp, LOCATION_MZONE) > 0 && duel.is_existing_target(f, tp, LOCATION_GRAVE, 0, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
    let g = duel.select_target(tp, f, tp, LOCATION_GRAVE, 0, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, g, 1, 0, 0);
    Ok(false)
}

fn operation(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let Some(tc) = duel.get_first_target() else { return Ok(()) };
    if tc.is_relate_to_effect(duel, e) && duel.special_summon(tc, 0, tp, tp, false, false, POS_FACEUP)? != 0 {
        let c = e.get_handler(duel);
        let fid = c.get_field_id(duel);
        tc.register_flag_effect_full(duel, ID, RESET_EVENT | RESETS_STANDARD, 0, 1, fid.into(), 0);
        let e1 = duel.create_effect(c);
        e1.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_CONTINUOUS);
        e1.set_code(duel, EVENT_PHASE + PHASE_END);
        e1.set_count_limit(duel, 1, 0);
        e1.set_property(duel, EFFECT_FLAG_IGNORE_IMMUNE);
        e1.set_label(duel, fid.into());
        e1.set_label_object_group(duel, Some(tc.into()));
        e1.set_condition_mut(duel, descon);
        e1.set_operation(duel, desop);
        duel.register_effect(e1, tp);
    }
    Ok(())
}

fn descon(duel: &mut Duel, a: &Args) -> bool {
    let tc = a.e.get_label_object_group(duel).and_then(|g| g.get_first());
    if tc.is_none_or(|tc| tc.get_flag_effect_label(duel, ID) != Some(a.e.get_label(duel))) {
        a.e.reset(duel);
        return false;
    }
    true
}

fn desop(duel: &mut Duel, a: &Args) -> Script<()> {
    if let Some(tc) = a.e.get_label_object_group(duel).and_then(|g| g.get_first()) {
        duel.destroy(tc, REASON_EFFECT)?;
    }
    Ok(())
}
