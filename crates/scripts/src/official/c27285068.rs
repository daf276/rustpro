// Theorealize Liberation
use crate::prelude::*;
use crate::testing::{Check, DECK, GRAVE, HAND, MZONE};

const ID: u32 = 27285068;

pub const LISTED_NAMES: &[u32] = &[CARD_MEDIUS_THE_PURE];

// Medius the Pure on the field to activate, Elfnote Power Patron (300 ATK / 200 DEF) in the Deck to add or Special Summon,
// a copy in the Graveyard and a non-Tuner monster on the field.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, GRAVE),
    (CARD_MEDIUS_THE_PURE, MZONE),
    (12375297, DECK),
    (12375297, DECK),
    (13597785, MZONE),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_TOHAND + CATEGORY_SEARCH + CATEGORY_SPECIAL_SUMMON);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_count_limit(duel, 1, ID);
    e1.set_condition(duel, |duel, a| {
        let filter = aux::faceup_filter(|c, duel| c.is_code(duel, CARD_MEDIUS_THE_PURE));
        duel.is_existing_matching_card(filter, a.tp, LOCATION_ONFIELD, 0, 1, None)
    });
    e1.set_target(duel, thsptg);
    e1.set_operation(duel, thspop);
    e1.set_hint_timing(duel, 0, TIMING_STANDBY_PHASE | TIMING_MAIN_END | TIMINGS_CHECK_MONSTER_E);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_type(duel, EFFECT_TYPE_IGNITION);
    e2.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e2.set_range(duel, LOCATION_GRAVE);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_cost(duel, Cost::self_banish);
    e2.set_target(duel, tunertg);
    e2.set_operation(duel, tunerop);
    c.register_effect(duel, e2);
}

fn thspfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8, sp_chk: bool) -> bool {
    c.is_attack(duel, &[300])
        && c.is_defense(duel, &[200])
        && (c.is_able_to_hand(duel) || (sp_chk && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)))
}

fn thsptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if chk == 0 {
        let sp_chk = duel.get_location_count(tp, LOCATION_MZONE) > 0;
        return Ok(duel.is_existing_matching_card(|c, duel| thspfilter(c, duel, e, tp, sp_chk), tp, LOCATION_DECK, 0, 1, None));
    }
    duel.set_possible_operation_info(0, CATEGORY_TOHAND, None, 1, tp, LOCATION_DECK as i32);
    duel.set_possible_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, tp, LOCATION_DECK as i32);
    Ok(false)
}

fn thspop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let sp_chk = duel.get_location_count(tp, LOCATION_MZONE) > 0;
    duel.hint(HINT_SELECTMSG, tp, aux::stringid(ID, 2));
    let g = duel.select_matching_card(tp, |c, duel| thspfilter(c, duel, e, tp, sp_chk), tp, LOCATION_DECK, 0, 1, 1, None)?;
    if let Some(sc) = g.get_first() {
        aux::to_hand_or_else(
            duel,
            sc,
            tp,
            |sc, duel| sp_chk && sc.is_can_be_special_summoned(duel, e, 0, tp, false, false),
            |duel| duel.special_summon(sc, 0, tp, tp, false, false, POS_FACEUP).map(drop),
            aux::stringid(ID, 3),
        )?;
    }
    Ok(())
}

fn tunerfilter(c: CardId, duel: &Duel) -> bool {
    c.is_faceup(duel) && !c.is_type(duel, TYPE_TUNER)
}

fn tunertg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_MZONE) && chkc.is_controler(duel, tp) && tunerfilter(chkc, duel));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(tunerfilter, tp, LOCATION_MZONE, 0, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TARGET);
    duel.select_target(tp, tunerfilter, tp, LOCATION_MZONE, 0, 1, 1, None)?;
    Ok(false)
}

fn tunerop(duel: &mut Duel, a: &Args) -> Script<()> {
    if let Some(tc) = duel.get_first_target()
        && tc.is_relate_to_effect(duel, a.e)
        && tc.is_faceup(duel)
    {
        let e1 = duel.create_effect(a.e.get_handler(duel));
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE);
        e1.set_code(duel, EFFECT_ADD_TYPE);
        e1.set_value(duel, TYPE_TUNER as i32);
        e1.set_reset(duel, RESETS_STANDARD_PHASE_END);
        tc.register_effect(duel, e1);
    }
    Ok(())
}
