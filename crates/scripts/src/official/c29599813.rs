// Purrely Pretty Memory
use crate::prelude::*;
use crate::testing::{Check, HAND, MZONE, OPPONENT};

const ID: u32 = 29599813;

// In the hand, and with Epurrely Beauty on the field to attach it and with Sangan on the opponent's side to take.
pub const SUPPORT: &[(u32, u16)] = &[(ID, HAND), (ID, HAND), (98049934, MZONE), (26202165, MZONE | OPPONENT)];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesGranted(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_RECOVER + CATEGORY_HANDES + CATEGORY_SPECIAL_SUMMON);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_hint_timing(duel, TIMING_END_PHASE, TIMING_END_PHASE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_target(duel, target);
    e1.set_operation(duel, operation);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_type(duel, EFFECT_TYPE_XMATERIAL + EFFECT_TYPE_IGNITION);
    e2.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e2.set_count_limit(duel, 1, 0);
    e2.set_condition(duel, |duel, a| a.e.get_handler(duel).is_set_card(duel, &[SET_PURRELY]));
    e2.set_cost(duel, atchcost);
    e2.set_target(duel, atchtg);
    e2.set_operation(duel, atchop);
    c.register_effect(duel, e2);
}

fn target(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(true);
    }
    duel.set_operation_info(0, CATEGORY_RECOVER, None, 0, PLAYER_ALL, 1000);
    duel.set_possible_operation_info(0, CATEGORY_HANDES, None, 0, a.tp, 1);
    duel.set_possible_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, a.tp, LOCATION_DECK as i32);
    Ok(false)
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_set_card(duel, &[SET_PURRELY]) && c.is_level(duel, &[1]) && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
}

fn operation(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    if duel.recover(tp, 1000, REASON_EFFECT)? == 0 || duel.recover(1 - tp, 1000, REASON_EFFECT)? == 0 {
        return Ok(());
    }
    if duel.is_existing_matching_card(CardId::is_discardable, tp, LOCATION_HAND, 0, 1, None)
        && duel.get_location_count(tp, LOCATION_MZONE) > 0
        && duel.is_existing_matching_card(|c, duel| spfilter(c, duel, e, tp), tp, LOCATION_DECK, 0, 1, None)
        && duel.select_yes_no(tp, aux::stringid(ID, 2))?
    {
        duel.break_effect()?;
        if duel.discard_hand(tp, CardId::is_discardable, 1, 1, REASON_EFFECT | REASON_DISCARD)? > 0 {
            duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
            let g = duel.select_matching_card(tp, |c, duel| spfilter(c, duel, e, tp), tp, LOCATION_DECK, 0, 1, 1, None)?;
            if !g.is_empty() {
                duel.special_summon(g, 0, tp, tp, false, false, POS_FACEUP)?;
            }
        }
    }
    Ok(())
}

fn atchfilter(c: CardId, duel: &Duel) -> bool {
    !c.is_type(duel, TYPE_TOKEN) && c.is_able_to_change_controler(duel)
}

fn atchcostfilter(c: CardId, duel: &Duel, tp: u8) -> bool {
    c.is_able_to_grave_as_cost(duel)
        && duel.is_existing_target(
            atchfilter,
            tp,
            0,
            LOCATION_ONFIELD,
            1,
            Some(c.get_equip_group(duel).union(c.into())),
        )
}

fn atchcost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let (c, tp) = (a.e.get_handler(duel), a.tp);
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(|g, duel| atchcostfilter(g, duel, tp), tp, LOCATION_ONFIELD, 0, 1, Some(c.into())));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TOGRAVE);
    let g = duel.select_matching_card(
        tp,
        |g, duel| atchcostfilter(g, duel, tp),
        tp,
        LOCATION_ONFIELD,
        0,
        1,
        1,
        Some(c.into()),
    )?;
    duel.send_to_grave(g, REASON_COST)?;
    Ok(false)
}

fn atchtg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_ONFIELD) && chkc.is_controler(duel, 1 - tp) && atchfilter(chkc, duel));
    }
    if chk == 0 {
        return Ok(a.e.get_handler(duel).is_type(duel, TYPE_XYZ) && duel.is_existing_target(atchfilter, tp, 0, LOCATION_ONFIELD, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TARGET);
    duel.select_target(tp, atchfilter, tp, 0, LOCATION_ONFIELD, 1, 1, None)?;
    Ok(false)
}

fn atchop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, c) = (a.e, a.e.get_handler(duel));
    let Some(tc) = duel.get_first_target() else { return Ok(()) };
    if c.is_relate_to_effect(duel, e) && tc.is_relate_to_effect(duel, e) && !tc.is_immune_to_effect(duel, e) {
        tc.cancel_to_grave(duel);
        duel.overlay_full(c, tc, true)?;
    }
    Ok(())
}
