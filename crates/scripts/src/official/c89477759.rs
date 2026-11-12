// Number C1000: Numerounius
use crate::prelude::*;
use crate::testing::{Check, EXTRA, HAND, MZONE, OPPONENT};

const ID: u32 = 89477759;
pub const XYZ_NUMBER: u32 = 1000;
const NUMEROUNIA: u32 = 15862758;

pub const LISTED_NAMES: &[u32] = &[NUMEROUNIA];

// On the field with Numerounia in the Extra Deck, the opponent's monsters to destroy and its Raigeki to destroy this card.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (NUMEROUNIA, EXTRA),
    (26202165, MZONE | OPPONENT),
    (13597785, MZONE | OPPONENT),
    (12580477, HAND | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
    Check::ResolvesEffect(aux::stringid(ID, 2)),
];

struct Procedure;

impl XyzProcedure for Procedure {
    const LV: Option<u32> = Some(12);
    const CT: usize = 5;
}

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    Xyz::add_procedure::<Procedure>(duel, c);
    c.enable_revive_limit(duel);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_DESTROY);
    e1.set_type(duel, EFFECT_TYPE_QUICK_O);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_hint_timing(duel, 0, TIMINGS_CHECK_MONSTER_E);
    e1.set_count_limit(duel, 1, 0);
    e1.set_cost(duel, Cost::detach_from_self::<1, 1>);
    e1.set_target(duel, destg);
    e1.set_operation(duel, desop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_DESTROY + CATEGORY_SPECIAL_SUMMON);
    e2.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_F);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_count_limit(duel, 1, 0);
    e2.set_code(duel, EVENT_PHASE | PHASE_BATTLE);
    e2.set_target(duel, bptg);
    e2.set_operation(duel, bpop);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 2));
    e3.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e3.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e3.set_property(duel, EFFECT_FLAG_DAMAGE_STEP + EFFECT_FLAG_DELAY);
    e3.set_code(duel, EVENT_TO_GRAVE);
    e3.set_condition(duel, spcon);
    e3.set_target(duel, sptg);
    e3.set_operation(duel, spop);
    c.register_effect(duel, e3);
}

fn destg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (c, tp) = (a.e.get_handler(duel), a.tp);
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(aux::TRUE, tp, LOCATION_MZONE, LOCATION_MZONE, 1, Some(c.into())));
    }
    let g = duel.get_matching_group(aux::TRUE, tp, LOCATION_MZONE, LOCATION_MZONE, Some(c.into()));
    duel.set_operation_info(0, CATEGORY_DESTROY, Some(g), 1, 0, 0);
    Ok(false)
}

fn desop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (c, tp) = (a.e.get_handler(duel), a.tp);
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_DESTROY);
    let g = duel.select_matching_card(tp, aux::TRUE, tp, LOCATION_MZONE, LOCATION_MZONE, 1, 1, Some(c.into()))?;
    if !g.is_empty() {
        duel.hint_selection(g);
        duel.destroy(g, REASON_EFFECT)?;
    }
    Ok(())
}

fn bptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (c, tp) = (a.e.get_handler(duel), a.tp);
    if chk == 0 {
        return Ok(true);
    }
    let sg = duel.get_matching_group(aux::TRUE, tp, LOCATION_MZONE, LOCATION_MZONE, Some(c.into()));
    duel.set_operation_info(0, CATEGORY_DESTROY, Some(sg), sg.len() as u8, 0, 0);
    duel.set_possible_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, 1 - tp, LOCATION_GRAVE as i32);
    Ok(false)
}

fn filter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_can_be_special_summoned_full(duel, e, 0, tp, false, false, POS_FACEUP_DEFENSE, tp, 0xff)
}

fn bpop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, c, tp) = (a.e, a.e.get_handler(duel), a.tp);
    let dg = duel.get_matching_group(aux::TRUE, tp, LOCATION_MZONE, LOCATION_MZONE, Some(c.into()));
    let ct = duel.destroy(dg, REASON_EFFECT)?;
    if ct == 0 || duel.get_location_count(tp, LOCATION_MZONE) < 1 {
        return Ok(());
    }
    let g = duel.get_matching_group(|tc, duel| filter(tc, duel, e, tp), tp, 0, LOCATION_GRAVE, None);
    if !g.is_empty() && duel.select_yes_no(tp, aux::stringid(ID, 3))? {
        duel.break_effect()?;
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
        let sg = g.select(duel, tp, 1, 1, None)?;
        duel.special_summon(sg, 0, tp, tp, false, false, POS_FACEUP_DEFENSE)?;
    }
    Ok(())
}

fn spcon(duel: &Duel, a: &Args) -> bool {
    let c = a.e.get_handler(duel);
    c.is_reason(duel, REASON_DESTROY)
        && a.rp != a.tp
        && c.is_reason(duel, REASON_EFFECT)
        && c.is_previous_location(duel, LOCATION_MZONE)
        && c.is_previous_controler(duel, a.tp)
        && c.get_overlay_count(duel) > 0
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_code(duel, NUMEROUNIA)
        && c.is_can_be_special_summoned(duel, e, 0x53b, tp, false, false)
        && duel.get_location_count_from_ex(tp, tp, None, c) > 0
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, c, tp) = (a.e, a.e.get_handler(duel), a.tp);
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(|tc, duel| spfilter(tc, duel, e, tp), tp, LOCATION_EXTRA, 0, 1, None));
    }
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, tp, LOCATION_EXTRA as i32);
    duel.set_operation_info(0, CATEGORY_LEAVE_GRAVE, Some(c.into()), 1, 0, 0);
    Ok(false)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, c, tp) = (a.e, a.e.get_handler(duel), a.tp);
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
    let g = duel.select_matching_card(tp, |tc, duel| spfilter(tc, duel, e, tp), tp, LOCATION_EXTRA, 0, 1, 1, None)?;
    if let Some(tc) = g.get_first()
        && duel.special_summon(tc, 0x53b, tp, tp, false, false, POS_FACEUP)? > 0
        && c.is_relate_to_effect(duel, e)
    {
        duel.overlay(tc, c)?;
    }
    Ok(())
}
