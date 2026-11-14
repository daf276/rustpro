// Kashtira Preparations
use crate::prelude::*;
use crate::testing::{Check, HAND, MZONE, REMOVED, SZONE};

const ID: u32 = 21639276;

pub const LISTED_SERIES: &[u16] = &[SET_KASHTIRA];
// Set copies with Kashtira Fenrir in the hand, banished and on the field.
pub const SUPPORT: &[(u32, u16)] = &[(ID, SZONE), (ID, SZONE), (32909498, HAND), (32909498, REMOVED), (32909498, MZONE)];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 0));
    e2.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e2.set_type(duel, EFFECT_TYPE_QUICK_O);
    e2.set_code(duel, EVENT_FREE_CHAIN);
    e2.set_range(duel, LOCATION_SZONE);
    e2.set_hint_timing(duel, 0, TIMING_END_PHASE);
    e2.set_count_limit(duel, 1, ID);
    e2.set_target(duel, sptg);
    e2.set_operation(duel, spop);
    c.register_effect(duel, e2);
    let e3a = duel.create_effect(c);
    e3a.set_type(duel, EFFECT_TYPE_CONTINUOUS + EFFECT_TYPE_FIELD);
    e3a.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE);
    e3a.set_code(duel, EVENT_CHAINING);
    e3a.set_range(duel, LOCATION_SZONE);
    e3a.set_operation(duel, chainreg);
    c.register_effect(duel, e3a);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 1));
    e3.set_category(duel, CATEGORY_REMOVE);
    e3.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e3.set_property(duel, EFFECT_FLAG_DELAY);
    e3.set_code(duel, EVENT_CHAIN_SOLVED);
    e3.set_range(duel, LOCATION_SZONE);
    e3.set_count_limit(duel, 1, (ID, 1));
    e3.set_condition(duel, rmvcond);
    e3.set_target(duel, rmvtg);
    e3.set_operation(duel, rmvop);
    c.register_effect(duel, e3);
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_set_card(duel, &[SET_KASHTIRA])
        && (c.is_location(duel, LOCATION_HAND) || c.is_faceup(duel))
        && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if chk == 0 {
        return Ok(duel.get_location_count(tp, LOCATION_MZONE) > 0
            && duel.is_existing_matching_card(|c, duel| spfilter(c, duel, e, tp), tp, LOCATION_HAND | LOCATION_REMOVED, 0, 1, None));
    }
    let description = e.get_description(duel);
    duel.hint(HINT_OPSELECTED, 1 - tp, description);
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, tp, (LOCATION_HAND | LOCATION_REMOVED) as i32);
    Ok(false)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    if duel.get_location_count(tp, LOCATION_MZONE) == 0 {
        return Ok(());
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
    let g = duel.select_matching_card(
        tp,
        |c, duel| spfilter(c, duel, e, tp),
        tp,
        LOCATION_HAND | LOCATION_REMOVED,
        0,
        1,
        1,
        None,
    )?;
    if !g.is_empty() {
        duel.special_summon(g, 0, tp, tp, false, false, POS_FACEUP)?;
    }
    Ok(())
}

fn chainreg(duel: &mut Duel, a: &Args) -> Script<()> {
    let c = a.e.get_handler(duel);
    if c.get_flag_effect(duel, ID) == 0 {
        c.register_flag_effect(duel, ID, RESET_EVENT | (RESETS_STANDARD & !RESET_TURN_SET) | RESET_CHAIN, 0, 1);
    }
    Ok(())
}

fn rmvcond(duel: &Duel, a: &Args) -> bool {
    let c = a.e.get_handler(duel);
    a.rp == 1 - a.tp
        && a.re.is_some_and(|re| re.is_trap_effect(duel))
        && c.is_status(duel, STATUS_EFFECT_ENABLED)
        && c.has_flag_effect(duel, ID)
}

fn rmvtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if chk == 0 {
        let filter = aux::faceup_filter(|c, duel| c.is_set_card(duel, &[SET_KASHTIRA]));
        let removable = |c: CardId, duel: &Duel| c.is_able_to_remove_full(duel, Some(tp), POS_FACEDOWN, REASON_EFFECT);
        return Ok(duel.is_existing_matching_card(filter, tp, LOCATION_MZONE, 0, 1, None)
            && duel.get_matching_group_count(removable, tp, 0, LOCATION_HAND, None) > 0);
    }
    let description = e.get_description(duel);
    duel.hint(HINT_OPSELECTED, 1 - tp, description);
    duel.set_operation_info(0, CATEGORY_REMOVE, None, 1, 1 - tp, LOCATION_HAND as i32);
    Ok(false)
}

fn rmvop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    let g = duel.get_field_group(tp, 0, LOCATION_HAND);
    if g.is_empty() {
        return Ok(());
    }
    duel.confirm_cards(tp, g)?;
    let removable = |c: CardId, duel: &Duel| c.is_able_to_remove_full(duel, Some(tp), POS_FACEDOWN, REASON_EFFECT);
    if g.is_exists(duel, removable, 1, None) {
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_REMOVE);
        let tc = g.filter_select(duel, tp, removable, 1, 1, None)?;
        if !tc.is_empty() {
            duel.remove(tc, POS_FACEDOWN, REASON_EFFECT)?;
            duel.shuffle_hand(1 - tp)?;
        }
    }
    Ok(())
}
