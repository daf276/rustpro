// Return of the Red-Eyes
use crate::prelude::*;
use crate::testing::{Check, GRAVE, HAND, MZONE, OPPONENT, SZONE};

const ID: u32 = 39387565;

pub const LISTED_SERIES: &[u16] = &[SET_RED_EYES];
// Set with Red-Eyes Black Dragons on the field and in the Graveyard to revive, and the opponent's Mystical Space Typhoon to
// destroy it.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, SZONE),
    (ID, SZONE),
    (CARD_REDEYES_B_DRAGON, MZONE),
    (CARD_REDEYES_B_DRAGON, GRAVE),
    (CARD_REDEYES_B_DRAGON, GRAVE),
    (5318639, HAND | OPPONENT),
    (5318639, HAND | OPPONENT),
];
pub const CHECKS: &[Check] = &[Check::SpecialSummonedCode(CARD_REDEYES_B_DRAGON)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_hint_timing(duel, 0, TIMING_END_PHASE);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e2.set_type(duel, EFFECT_TYPE_QUICK_O);
    e2.set_code(duel, EVENT_FREE_CHAIN);
    e2.set_range(duel, LOCATION_SZONE);
    e2.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e2.set_count_limit(duel, 1, ID);
    e2.set_condition(duel, spcon1);
    e2.set_target(duel, sptg1);
    e2.set_operation(duel, spop1);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e3.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e3.set_code(duel, EVENT_TO_GRAVE);
    e3.set_property(duel, EFFECT_FLAG_CARD_TARGET + EFFECT_FLAG_DAMAGE_STEP + EFFECT_FLAG_DELAY);
    e3.set_count_limit(duel, 1, (ID, 1));
    e3.set_condition(duel, spcon2);
    e3.set_target(duel, sptg2);
    e3.set_operation(duel, spop2);
    c.register_effect(duel, e3);
}

fn spcon1(duel: &Duel, a: &Args) -> bool {
    let filter = aux::faceup_filter(|c, duel| c.is_set_card(duel, &[SET_RED_EYES]));
    duel.is_existing_matching_card(filter, a.tp, LOCATION_MZONE, 0, 1, None)
}

fn spfilter1(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_type(duel, TYPE_NORMAL) && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
}

fn spfilter2(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_set_card(duel, &[SET_RED_EYES]) && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
}

fn graveyard_target(
    duel: &mut Duel,
    a: &Args,
    chk: i32,
    chkc: Option<CardId>,
    filter: fn(CardId, &Duel, EffectId, u8) -> bool,
) -> Script<bool> {
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

fn sptg1(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    graveyard_target(duel, a, chk, chkc, spfilter1)
}

fn sptg2(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    graveyard_target(duel, a, chk, chkc, spfilter2)
}

fn spop1(duel: &mut Duel, a: &Args) -> Script<()> {
    if !a.e.get_handler(duel).is_relate_to_effect(duel, a.e) {
        return Ok(());
    }
    spop2(duel, a)
}

fn spcon2(_: &Duel, a: &Args) -> bool {
    a.r & (REASON_DESTROY + REASON_EFFECT) == REASON_DESTROY + REASON_EFFECT && a.rp == 1 - a.tp
}

fn spop2(duel: &mut Duel, a: &Args) -> Script<()> {
    if let Some(tc) = duel.get_first_target()
        && tc.is_relate_to_effect(duel, a.e)
    {
        duel.special_summon(tc, 0, a.tp, a.tp, false, false, POS_FACEUP)?;
    }
    Ok(())
}
