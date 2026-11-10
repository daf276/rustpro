// DoomZ Destruction
use crate::prelude::*;
use crate::testing::{Check, DECK, HAND, MZONE, OPPONENT, SZONE};

const ID: u32 = 80320877;

// Set, to equip DoomZ Break - Diactorus (a DoomZ Xyz) on the field, with DoomZ V Five - Amalthe in the Deck to equip and an
// opponent's Mystical Space Typhoon to destroy it.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, SZONE),
    (ID, SZONE),
    (31010081, MZONE),
    (31010081, MZONE),
    (28877382, DECK),
    (28877382, DECK),
    (5318639, HAND | OPPONENT),
    (5318639, HAND | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_EQUIP);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_count_limit(duel, 1, ID);
    e1.set_hint_timing(duel, 0, TIMING_STANDBY_PHASE | TIMING_MAIN_END | TIMINGS_CHECK_MONSTER_E);
    e1.set_cost(duel, aux::remain_field_cost);
    e1.set_target(duel, target);
    e1.set_operation(duel, operation);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_EQUIP);
    e2.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e2.set_property(duel, EFFECT_FLAG_CARD_TARGET + EFFECT_FLAG_DELAY);
    e2.set_code(duel, EVENT_DESTROYED);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_condition(duel, |_, a| a.r & REASON_EFFECT > 0);
    e2.set_target(duel, eqptg);
    e2.set_operation(duel, eqpop);
    c.register_effect(duel, e2);
}

fn xyzfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_DOOMZ]) && c.is_type(duel, TYPE_XYZ) && c.is_faceup(duel)
}

fn target(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_controler(duel, tp) && chkc.is_location(duel, LOCATION_MZONE) && xyzfilter(chkc, duel));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(xyzfilter, tp, LOCATION_MZONE, 0, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_EQUIP);
    duel.select_target(tp, xyzfilter, tp, LOCATION_MZONE, 0, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_EQUIP, Some(a.e.get_handler(duel).into()), 1, tp, 0);
    Ok(false)
}

fn operation(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if !c.is_relate_to_effect(duel, e) || c.is_status(duel, STATUS_LEAVE_CONFIRMED) {
        return Ok(());
    }
    let tc = duel.get_first_target();
    if let Some(tc) = tc.filter(|tc| tc.is_relate_to_effect(duel, e) && tc.is_faceup(duel))
        && duel.equip(tp, c, tc)?
    {
        let e1 = duel.create_effect(c);
        e1.set_type(duel, EFFECT_TYPE_FIELD);
        e1.set_code(duel, EFFECT_CANNOT_TO_HAND);
        e1.set_range(duel, LOCATION_SZONE);
        e1.set_target_range(duel, 0, LOCATION_DECK);
        e1.set_reset(duel, RESET_EVENT | RESETS_STANDARD);
        c.register_effect(duel, e1);
        add_equip_limit(duel, c, tc);
    } else {
        c.cancel_to_grave_full(duel, false);
    }
    Ok(())
}

fn add_equip_limit(duel: &mut Duel, ec: CardId, tc: CardId) {
    let e = duel.create_effect(ec);
    e.set_type(duel, EFFECT_TYPE_SINGLE);
    e.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE);
    e.set_code(duel, EFFECT_EQUIP_LIMIT);
    e.set_value(duel, Value::Card(super::c38423248::equip_limit));
    e.set_label_object_group(duel, Some(tc.into()));
    e.set_reset(duel, RESET_EVENT | RESETS_STANDARD);
    ec.register_effect(duel, e);
}

fn eqfilter(c: CardId, duel: &Duel, tp: u8) -> bool {
    c.is_set_card(duel, &[SET_DOOMZ]) && c.is_monster(duel) && c.check_unique_on_field(duel, tp) && !c.is_forbidden(duel)
}

fn eqptg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_MZONE) && chkc.is_faceup(duel));
    }
    if chk == 0 {
        return Ok(duel.get_location_count(tp, LOCATION_SZONE) > 0
            && duel.is_existing_target(CardId::is_faceup, tp, LOCATION_MZONE, LOCATION_MZONE, 1, None)
            && duel.is_existing_matching_card(|c, duel| eqfilter(c, duel, tp), tp, LOCATION_DECK, 0, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_EQUIP);
    duel.select_target(tp, CardId::is_faceup, tp, LOCATION_MZONE, LOCATION_MZONE, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_EQUIP, None, 1, tp, LOCATION_DECK as i32);
    Ok(false)
}

fn eqpop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let Some(tc) = duel.get_first_target() else { return Ok(()) };
    if !(tc.is_faceup(duel) && tc.is_relate_to_effect(duel, e) && duel.get_location_count(tp, LOCATION_SZONE) > 0) {
        return Ok(());
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_EQUIP);
    let g = duel.select_matching_card(tp, |c, duel| eqfilter(c, duel, tp), tp, LOCATION_DECK, 0, 1, 1, None)?;
    if let Some(ec) = g.get_first()
        && duel.equip(tp, ec, tc)?
    {
        add_equip_limit(duel, ec, tc);
    }
    Ok(())
}
