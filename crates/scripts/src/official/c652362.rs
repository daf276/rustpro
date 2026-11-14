// Alien Ammonite
use crate::prelude::*;
use crate::testing::{Check, GRAVE, HAND};

const ID: u32 = 652362;

pub const LISTED_SERIES: &[u16] = &[SET_ALIEN];
// Copies in the hand to Normal Summon and in the Graveyard to Special Summon.
pub const SUPPORT: &[(u32, u16)] = &[(ID, HAND), (ID, HAND), (ID, HAND), (ID, GRAVE), (ID, GRAVE)];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0)), Check::SpecialSummoned];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e1.set_code(duel, EVENT_SUMMON_SUCCESS);
    e1.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e1.set_target(duel, sptg);
    e1.set_operation(duel, spop);
    c.register_effect(duel, e1);
}

fn filter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    let lv = c.get_level(duel);
    lv > 0 && lv <= 4 && c.is_set_card(duel, &[SET_ALIEN]) && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_GRAVE) && chkc.is_controler(duel, tp) && filter(chkc, duel, e, tp));
    }
    if chk == 0 {
        return Ok(duel.get_location_count(tp, LOCATION_MZONE) > 0
            && duel.is_existing_target(|c, duel| filter(c, duel, e, tp), tp, LOCATION_GRAVE, 0, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
    let g = duel.select_target(tp, |c, duel| filter(c, duel, e, tp), tp, LOCATION_GRAVE, 0, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, g, 1, 0, 0);
    Ok(false)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
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
    unported!(duel, tc.is_none());
    if tc.is_some_and(|tc| tc.get_flag_effect_label(duel, ID) != Some(a.e.get_label(duel))) {
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
