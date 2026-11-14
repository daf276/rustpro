// Gladiator Beast Darius
use crate::prelude::*;
use crate::testing::{Check, DECK, GRAVE, MZONE, OPPONENT};

const ID: u32 = 25924653;

pub const LISTED_SERIES: &[u16] = &[SET_GLADIATOR_BEAST];
pub const LISTED_NAMES: &[u32] = &[ID];

// On the field with Gyzarus, Bestiari and Hoplomus to battle Sangan and tag out for each other, Darius bringing back Hoplomus
// from the GY.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (48156348, MZONE),
    (41470137, MZONE),
    (41470137, MZONE),
    (4253484, MZONE),
    (ID, DECK),
    (ID, DECK),
    (ID, DECK),
    (41470137, DECK),
    (4253484, GRAVE),
    (26202165, MZONE | OPPONENT),
    (26202165, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e1.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e1.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e1.set_condition(duel, aux::gbspcon);
    e1.set_target(duel, gysptg);
    e1.set_operation(duel, gyspop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e2.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e2.set_code(duel, EVENT_PHASE | PHASE_BATTLE);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_condition(duel, |duel, a| a.e.get_handler(duel).get_battled_group_count(duel) > 0);
    e2.set_cost(duel, Cost::self_to_deck);
    e2.set_target(duel, dsptg);
    e2.set_operation(duel, dspop);
    c.register_effect(duel, e2);
}

fn gyspfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_set_card(duel, &[SET_GLADIATOR_BEAST]) && c.is_can_be_special_summoned(duel, e, 104, tp, false, false)
}

fn gysptg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_GRAVE) && chkc.is_controler(duel, tp) && gyspfilter(chkc, duel, e, tp));
    }
    let filter = |c, duel: &Duel| gyspfilter(c, duel, e, tp);
    if chk == 0 {
        return Ok(duel.get_location_count(tp, LOCATION_MZONE) > 0 && duel.is_existing_target(filter, tp, LOCATION_GRAVE, 0, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
    let g = duel.select_target(tp, filter, tp, LOCATION_GRAVE, 0, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, g, 1, 0, 0);
    Ok(false)
}

fn gyspop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if let Some(tc) = duel.get_first_target()
        && tc.is_relate_to_effect(duel, e)
        && duel.special_summon_step(tc, 104, tp, tp, false, false, POS_FACEUP)?
    {
        tc.negate_effects(duel, c, 0, false);
        if c.is_faceup(duel) && c.is_relate_to_effect(duel, e) {
            c.set_card_target(duel, tc);
            let e1 = duel.create_effect(c);
            e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_CONTINUOUS);
            e1.set_code(duel, EVENT_LEAVE_FIELD_P);
            e1.set_operation(duel, regop);
            e1.set_reset(duel, RESET_EVENT | RESETS_STANDARD);
            c.register_effect(duel, e1);
        }
    }
    duel.special_summon_complete()?;
    Ok(())
}

fn regop(duel: &mut Duel, a: &Args) -> Script<()> {
    let c = a.e.get_handler(duel);
    if !c.get_first_card_target(duel).is_some_and(|tc| tc.is_location(duel, LOCATION_MZONE)) {
        return Ok(());
    }
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_CONTINUOUS);
    e1.set_code(duel, EVENT_LEAVE_FIELD);
    e1.set_operation(duel, tdop);
    c.register_effect(duel, e1);
    Ok(())
}

fn tdop(duel: &mut Duel, a: &Args) -> Script<()> {
    if let Some(tc) = a.e.get_handler(duel).get_first_card_target(duel)
        && tc.is_location(duel, LOCATION_MZONE)
    {
        duel.send_to_deck(tc, PLAYER_NONE, SEQ_DECKSHUFFLE, REASON_EFFECT)?;
    }
    a.e.reset(duel);
    Ok(())
}

fn dspfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_set_card(duel, &[SET_GLADIATOR_BEAST]) && c.is_can_be_special_summoned(duel, e, 104, tp, false, false) && !c.is_code(duel, ID)
}

fn dsptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if chk == 0 {
        let c = e.get_handler(duel);
        return Ok(duel.get_mzone_count(tp, Some(c.into())) > 0
            && duel.is_existing_matching_card(|c, duel| dspfilter(c, duel, e, tp), tp, LOCATION_DECK, 0, 1, None));
    }
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, tp, LOCATION_DECK as i32);
    Ok(false)
}

fn dspop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    if duel.get_location_count(tp, LOCATION_MZONE) <= 0 {
        return Ok(());
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
    let g = duel.select_matching_card(tp, |c, duel| dspfilter(c, duel, e, tp), tp, LOCATION_DECK, 0, 1, 1, None)?;
    if let Some(sc) = g.get_first()
        && duel.special_summon(sc, 104, tp, tp, false, false, POS_FACEUP)? > 0
    {
        let code = sc.get_original_code(duel);
        sc.register_flag_effect(duel, code, RESET_EVENT | RESETS_STANDARD_DISABLE, 0, 0);
    }
    Ok(())
}
