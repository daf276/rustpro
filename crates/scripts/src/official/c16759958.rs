// Aromaseraphy Angelica
use crate::prelude::*;
use crate::testing::{Check, GRAVE, HAND, MZONE};

const ID: u32 = 16759958;

pub const LISTED_SERIES: &[u16] = &[SET_AROMA];
// Aromage Bergamot in the Graveyard to gain LP from and on the field for the condition, copies in the hand and Graveyard, and
// Aromalilith Rosalina in the hand to gain LP first.
pub const SUPPORT: &[(u32, u16)] = &[(ID, HAND), (ID, GRAVE), (85967160, GRAVE), (85967160, MZONE), (24689197, HAND)];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_RECOVER);
    e1.set_type(duel, EFFECT_TYPE_QUICK_O);
    e1.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_range(duel, LOCATION_HAND);
    e1.set_count_limit(duel, 1, ID);
    e1.set_cost(duel, Cost::self_discard);
    e1.set_target(duel, rectg);
    e1.set_operation(duel, recop);
    e1.set_hint_timing(duel, 0, TIMING_MAIN_END | TIMING_END_PHASE);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e2.set_type(duel, EFFECT_TYPE_IGNITION);
    e2.set_range(duel, LOCATION_GRAVE);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_condition(duel, spcon);
    e2.set_target(duel, stock::special_summon_self_target_player_0);
    e2.set_operation(duel, spop);
    c.register_effect(duel, e2);
}

fn recfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_AROMA]) && c.get_attack(duel) > 0
}

fn rectg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_GRAVE) && chkc.is_controler(duel, tp) && recfilter(chkc, duel));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(recfilter, tp, LOCATION_GRAVE, 0, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TARGET);
    let g = duel.select_target(tp, recfilter, tp, LOCATION_GRAVE, 0, 1, 1, None)?;
    let atk = g.and_then(|g| g.get_first()).map_or(0, |tc| tc.get_attack(duel));
    duel.set_operation_info(0, CATEGORY_RECOVER, None, 0, tp, atk);
    Ok(false)
}

fn recop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    if let Some(tc) = duel.get_first_target()
        && tc.is_relate_to_effect(duel, a.e)
        && tc.get_attack(duel) > 0
    {
        let atk = tc.get_attack(duel);
        duel.recover(tp, atk as u32, REASON_EFFECT)?;
    }
    Ok(())
}

fn spcon(duel: &Duel, a: &Args) -> bool {
    let tp = a.tp;
    duel.get_lp(tp) > duel.get_lp(1 - tp)
        && duel.is_existing_matching_card(
            aux::faceup_filter(|c, duel| c.is_set_card(duel, &[SET_AROMA])),
            tp,
            LOCATION_MZONE,
            0,
            1,
            None,
        )
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if c.is_relate_to_effect(duel, e) && duel.special_summon(c, 0, tp, tp, false, false, POS_FACEUP)? > 0 {
        let e1 = duel.create_effect(c);
        e1.set_description(duel, 3300);
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_code(duel, EFFECT_LEAVE_FIELD_REDIRECT);
        e1.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE + EFFECT_FLAG_CLIENT_HINT);
        e1.set_reset(duel, RESET_EVENT | RESETS_REDIRECT);
        e1.set_value(duel, LOCATION_REMOVED as i32);
        c.register_effect_full(duel, e1, true);
    }
    Ok(())
}
