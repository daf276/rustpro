// Blast with Chain
use crate::prelude::*;
use crate::testing::{Check, HAND, MZONE, OPPONENT, SZONE};

const ID: u32 = 98239899;

// In the hand and Set, to be activated on Mokey Mokey, and the opponent's Mystical Space Typhoon to destroy it while equipped.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, SZONE),
    (27288416, MZONE),
    (27288416, MZONE),
    (5318639, SZONE | OPPONENT),
    (5318639, SZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(0), Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_category(duel, CATEGORY_EQUIP);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_hint_timing(duel, TIMING_DAMAGE_STEP, TIMING_DAMAGE_STEP);
    e1.set_property(duel, EFFECT_FLAG_CARD_TARGET + EFFECT_FLAG_DAMAGE_STEP);
    e1.set_condition(duel, aux::stat_change_damage_step_condition);
    e1.set_cost(duel, aux::remain_field_cost);
    e1.set_target(duel, target);
    e1.set_operation(duel, operation);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 0));
    e2.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_F);
    e2.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e2.set_code(duel, EVENT_LEAVE_FIELD);
    e2.set_condition(duel, descon);
    e2.set_target(duel, destg);
    e2.set_operation(duel, desop);
    c.register_effect(duel, e2);
}

fn target(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if let Some(chkc) = chkc {
        return Ok(chkc.is_controler(duel, tp) && chkc.is_location(duel, LOCATION_MZONE) && chkc.is_faceup(duel));
    }
    if chk == 0 {
        return Ok(e.is_has_type(duel, EFFECT_TYPE_ACTIVATE) && duel.is_existing_target(CardId::is_faceup, tp, LOCATION_MZONE, 0, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_EQUIP);
    duel.select_target(tp, CardId::is_faceup, tp, LOCATION_MZONE, 0, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_EQUIP, Some(e.get_handler(duel).into()), 1, 0, 0);
    Ok(false)
}

fn operation(duel: &mut Duel, a: &Args) -> Script<()> {
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
    {
        duel.equip(tp, c, tc)?;
        let e1 = duel.create_effect(c);
        e1.set_type(duel, EFFECT_TYPE_EQUIP);
        e1.set_code(duel, EFFECT_UPDATE_ATTACK);
        e1.set_value(duel, 500);
        e1.set_reset(duel, RESET_EVENT | RESETS_STANDARD);
        c.register_effect(duel, e1);
        let e2 = duel.create_effect(c);
        e2.set_type(duel, EFFECT_TYPE_SINGLE);
        e2.set_code(duel, EFFECT_EQUIP_LIMIT);
        e2.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE);
        e2.set_value(
            duel,
            Value::Card(|duel, e, c| i32::from(c.is_controler(duel, e.get_owner_player(duel)))),
        );
        e2.set_reset(duel, RESET_EVENT | RESETS_STANDARD);
        c.register_effect(duel, e2);
    } else {
        c.cancel_to_grave_full(duel, false);
    }
    Ok(())
}

fn descon(duel: &Duel, a: &Args) -> bool {
    a.r & (REASON_DESTROY | REASON_EFFECT) == (REASON_DESTROY | REASON_EFFECT) && a.e.get_handler(duel).get_equip_target(duel).is_some()
}

fn destg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_on_field(duel));
    }
    if chk == 0 {
        return Ok(true);
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_DESTROY);
    let g = duel.select_target(tp, aux::TRUE, tp, LOCATION_ONFIELD, LOCATION_ONFIELD, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_DESTROY, g, g.map_or(0, |g| g.len() as u8), 0, 0);
    Ok(false)
}

fn desop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tc = duel.get_first_target();
    if let Some(tc) = tc
        && tc.is_relate_to_effect(duel, a.e)
    {
        duel.destroy(tc, REASON_EFFECT)?;
    }
    Ok(())
}
