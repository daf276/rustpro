// Malefic Force
use crate::prelude::*;
use crate::testing::{Check, DECK, GRAVE, HAND, MZONE};

const ID: u32 = 68721020;

// In the hand to equip and in the Graveyard for the second effect, with Malefic monsters in the Graveyard to banish and in the Deck to
// search, and Malefic World in the hand for the Field Zone.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, GRAVE),
    (27564031, HAND),
    (48829461, GRAVE),
    (74509280, GRAVE),
    (48829461, DECK),
    (74509280, MZONE),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
    Check::ResolvesEffect(aux::stringid(ID, 2)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_EQUIP);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_count_limit(duel, 1, (ID, 0));
    e1.set_cost(duel, aux::remain_field_cost);
    e1.set_target(duel, eqtg);
    e1.set_operation(duel, eqop);
    e1.set_hint_timing(duel, 0, TIMING_STANDBY_PHASE | TIMING_MAIN_END | TIMINGS_CHECK_MONSTER_E);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_DESTROY);
    e2.set_type(duel, EFFECT_TYPE_QUICK_O);
    e2.set_code(duel, EVENT_FREE_CHAIN);
    e2.set_range(duel, LOCATION_GRAVE);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_condition(duel, |duel, _| {
        duel.is_existing_matching_card(aux::TRUE, 0, LOCATION_FZONE, LOCATION_FZONE, 1, None)
    });
    e2.set_cost(duel, descost);
    e2.set_target(duel, destg);
    e2.set_operation(duel, desop);
    e2.set_hint_timing(duel, 0, TIMING_STANDBY_PHASE | TIMING_MAIN_END | TIMINGS_CHECK_MONSTER_E);
    c.register_effect(duel, e2);
}

fn eqtg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_MZONE) && chkc.is_faceup(duel));
    }
    if chk == 0 {
        return Ok(e.is_has_type(duel, EFFECT_TYPE_ACTIVATE)
            && duel.is_existing_target(CardId::is_faceup, tp, LOCATION_MZONE, LOCATION_MZONE, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_EQUIP);
    duel.select_target(tp, CardId::is_faceup, tp, LOCATION_MZONE, LOCATION_MZONE, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_EQUIP, Some(e.get_handler(duel).into()), 1, tp, 0);
    Ok(false)
}

fn eqop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if !c.is_location(duel, LOCATION_SZONE) || !c.is_relate_to_effect(duel, e) || c.is_status(duel, STATUS_LEAVE_CONFIRMED) {
        return Ok(());
    }
    let tc = duel.get_first_target();
    unported!(duel, tc.is_none());
    if let Some(tc) = tc
        && tc.is_relate_to_effect(duel, e)
        && tc.is_faceup(duel)
        && duel.equip(tp, c, tc)?
    {
        let e0 = duel.create_effect(c);
        e0.set_type(duel, EFFECT_TYPE_SINGLE);
        e0.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE);
        e0.set_code(duel, EFFECT_EQUIP_LIMIT);
        e0.set_value(duel, 1);
        e0.set_reset(duel, RESET_EVENT | RESETS_STANDARD);
        c.register_effect(duel, e0);
        let e1 = duel.create_effect(c);
        e1.set_type(duel, EFFECT_TYPE_EQUIP);
        e1.set_code(duel, EFFECT_IMMUNE_EFFECT);
        e1.set_value(duel, Value::Effect(|duel, _, te| te.is_spell_effect(duel)));
        e1.set_reset(duel, RESET_EVENT | RESETS_STANDARD);
        c.register_effect(duel, e1);
        let e2 = duel.create_effect(c);
        e2.set_description(duel, aux::stringid(ID, 2));
        e2.set_category(duel, CATEGORY_DESTROY + CATEGORY_TOHAND + CATEGORY_SEARCH);
        e2.set_type(duel, EFFECT_TYPE_QUICK_O);
        e2.set_code(duel, EVENT_FREE_CHAIN);
        e2.set_range(duel, LOCATION_SZONE);
        e2.set_count_limit(duel, 1, 0);
        e2.set_target(duel, desthtg);
        e2.set_operation(duel, desthop);
        e2.set_reset(duel, RESET_EVENT | RESETS_STANDARD);
        e2.set_hint_timing(duel, 0, TIMING_STANDBY_PHASE | TIMING_MAIN_END | TIMINGS_CHECK_MONSTER_E);
        c.register_effect(duel, e2);
    } else {
        c.cancel_to_grave_full(duel, false);
    }
    Ok(())
}

fn thfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_MALEFIC]) && c.is_able_to_hand(duel)
}

fn desthtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(thfilter, tp, LOCATION_DECK, 0, 1, None));
    }
    duel.set_operation_info(0, CATEGORY_DESTROY, Some(a.e.get_handler(duel).into()), 1, tp, 0);
    duel.set_operation_info(0, CATEGORY_TOHAND, None, 1, tp, LOCATION_DECK as i32);
    Ok(false)
}

fn desthop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if c.is_relate_to_effect(duel, e) && duel.destroy(c, REASON_EFFECT)? > 0 {
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_ATOHAND);
        let g = duel.select_matching_card(tp, thfilter, tp, LOCATION_DECK, 0, 1, 1, None)?;
        if !g.is_empty() {
            duel.send_to_hand(g, PLAYER_NONE, REASON_EFFECT)?;
            duel.confirm_cards(1 - tp, g)?;
        }
    }
    Ok(())
}

fn descostfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_MALEFIC]) && c.is_able_to_remove_as_cost(duel)
}

fn descost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let (c, tp) = (a.e.get_handler(duel), a.tp);
    if chk == 0 {
        return Ok(
            c.is_able_to_remove_as_cost(duel) && duel.is_existing_matching_card(descostfilter, tp, LOCATION_GRAVE, 0, 1, Some(c.into()))
        );
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_REMOVE);
    let mut g = duel.select_matching_card(tp, descostfilter, tp, LOCATION_GRAVE, 0, 1, 1, Some(c.into()))?;
    g.insert(c);
    duel.remove(g, POS_FACEUP, REASON_COST)?;
    Ok(false)
}

fn destg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    let g = duel.get_field_group(tp, LOCATION_MZONE, LOCATION_MZONE);
    if chk == 0 {
        return Ok(!g.is_empty());
    }
    duel.set_operation_info(0, CATEGORY_DESTROY, Some(g), g.len() as u8, tp, 0);
    Ok(false)
}

fn desop(duel: &mut Duel, a: &Args) -> Script<()> {
    let g = duel.get_field_group(a.tp, LOCATION_MZONE, LOCATION_MZONE);
    if !g.is_empty() {
        duel.destroy(g, REASON_EFFECT)?;
    }
    Ok(())
}
