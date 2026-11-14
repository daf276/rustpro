// Service Puppet Play
use crate::prelude::*;
use crate::testing::{Check, GRAVE, MZONE, SZONE};

const ID: u32 = 36400569;

pub const LISTED_SERIES: &[u16] = &[SET_GIMMICK_PUPPET];

// Set and in the Graveyard, with Gigantes Doll (a Gimmick Puppet Xyz) on the field and in the Graveyard to summon.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, SZONE),
    (ID, SZONE),
    (ID, GRAVE),
    (7593748, MZONE),
    (7593748, MZONE),
    (7593748, GRAVE),
    (7593748, GRAVE),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_CONTROL);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_hint_timing(duel, 0, TIMING_MAIN_END | TIMINGS_CHECK_MONSTER);
    e1.set_count_limit(duel, 1, ID);
    e1.set_target(duel, target);
    e1.set_operation(duel, activate);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e2.set_type(duel, EFFECT_TYPE_QUICK_O);
    e2.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e2.set_code(duel, EVENT_FREE_CHAIN);
    e2.set_range(duel, LOCATION_GRAVE);
    e2.set_hint_timing(duel, 0, TIMING_MAIN_END | TIMINGS_CHECK_MONSTER_E);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_condition(duel, |duel, a| {
        aux::exccon(duel, a) && duel.is_existing_matching_card(gpxyzfilter, a.tp, LOCATION_MZONE, 0, 1, None)
    });
    e2.set_cost(duel, Cost::self_banish);
    e2.set_target(duel, sptg);
    e2.set_operation(duel, spop);
    c.register_effect(duel, e2);
}

fn gpxyzfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_GIMMICK_PUPPET]) && c.is_type(duel, TYPE_XYZ) && c.is_faceup(duel)
}

fn target(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_MZONE) && chkc.is_controler(duel, 1 - tp) && chkc.is_controler_can_be_changed(duel));
    }
    let ct = duel.get_matching_group_count(gpxyzfilter, tp, LOCATION_MZONE, 0, None) as i32;
    if chk == 0 {
        return Ok(ct > 0 && duel.is_existing_target(CardId::is_controler_can_be_changed, tp, 0, LOCATION_MZONE, 1, None));
    }
    let ct = ct.min(duel.get_location_count_full(tp, LOCATION_MZONE, tp, LOCATION_REASON_CONTROL, 0xff));
    unported!(duel, ct < 1);
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_CONTROL);
    let g = duel.select_target(
        tp,
        CardId::is_controler_can_be_changed,
        tp,
        0,
        LOCATION_MZONE,
        1,
        ct.max(1) as u8,
        None,
    )?;
    let count = g.map_or(0, |g| g.len() as u8);
    duel.set_operation_info(0, CATEGORY_CONTROL, g, count, tp, 0);
    Ok(false)
}

fn activate(duel: &mut Duel, a: &Args) -> Script<()> {
    let tg = duel.get_target_cards(a.e).unwrap_or_default();
    if !tg.is_empty() {
        let reason_player = duel.get_reason_player();
        duel.get_control_full(tg, a.tp, PHASE_END as u16, 1, 0xff, reason_player)?;
    }
    Ok(())
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8, ftpl: i32, ftopp: i32) -> bool {
    c.is_type(duel, TYPE_XYZ)
        && ((ftpl > 0 && c.is_can_be_special_summoned_full(duel, e, 0, tp, false, false, POS_FACEUP_DEFENSE, tp, 0xff))
            || (ftopp > 0 && c.is_can_be_special_summoned_full(duel, e, 0, tp, false, false, POS_FACEUP_DEFENSE, 1 - tp, 0xff)))
}

fn opponent_count(duel: &Duel, tp: u8) -> i32 {
    duel.get_location_count_full(1 - tp, LOCATION_MZONE, tp, LOCATION_REASON_TOFIELD, 0xff)
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let ftpl = duel.get_location_count(tp, LOCATION_MZONE);
    let ftopp = opponent_count(duel, tp);
    let filter = |c, duel: &Duel| spfilter(c, duel, e, tp, ftpl, ftopp);
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_GRAVE) && filter(chkc, duel));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(filter, tp, LOCATION_GRAVE, LOCATION_GRAVE, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
    let g = duel.select_target(tp, filter, tp, LOCATION_GRAVE, LOCATION_GRAVE, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, g, 1, tp, 0);
    Ok(false)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let Some(tc) = duel.get_first_target().filter(|tc| tc.is_relate_to_effect(duel, e)) else {
        return Ok(());
    };
    let b1 = duel.get_location_count(tp, LOCATION_MZONE) > 0
        && tc.is_can_be_special_summoned_full(duel, e, 0, tp, false, false, POS_FACEUP_DEFENSE, tp, 0xff);
    let b2 =
        opponent_count(duel, tp) > 0 && tc.is_can_be_special_summoned_full(duel, e, 0, tp, false, false, POS_FACEUP_DEFENSE, 1 - tp, 0xff);
    if !(b1 || b2) {
        return Ok(());
    }
    let op = if b1 && b2 {
        duel.select_effect(tp, &[(b1, aux::stringid(ID, 2)), (b2, aux::stringid(ID, 3))])?
    } else if b1 {
        Some(1)
    } else {
        Some(2)
    };
    let target_player = if op == Some(1) { tp } else { 1 - tp };
    duel.special_summon(tc, 0, tp, target_player, false, false, POS_FACEUP_DEFENSE)?;
    Ok(())
}
