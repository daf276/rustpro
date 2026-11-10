// Sales Ban
use crate::prelude::*;
use crate::testing::{Check, HAND};

const ID: u32 = 64964750;

pub const SUPPORT: &[(u32, u16)] = &[(ID, HAND)];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_count_limit_full(duel, 1, ID, EFFECT_COUNT_CODE_OATH);
    e1.set_target(duel, target);
    e1.set_operation(duel, activate);
    c.register_effect(duel, e1);
}

fn target(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(true);
    }
    let tp = a.tp;
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_CODE);
    let ac = duel.announce_card(tp, &[])?;
    duel.set_target_param(ac);
    duel.set_operation_info(0, CATEGORY_ANNOUNCE, None, 0, tp, ANNOUNCE_CARD as i32);
    Ok(false)
}

fn activate(duel: &mut Duel, a: &Args) -> Script<()> {
    let (c, tp) = (a.e.get_handler(duel), a.tp);
    let ac: u32 = duel.get_chain_info(0, CHAININFO_TARGET_PARAM);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 1));
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_property(duel, EFFECT_FLAG_PLAYER_TARGET + EFFECT_FLAG_CLIENT_HINT);
    e1.set_code(duel, EFFECT_CANNOT_ACTIVATE);
    e1.set_target_range(duel, 0, 1);
    e1.set_label(duel, i64::from(ac));
    e1.set_value(
        duel,
        Value::EffectPlayer(|duel, e, re, _| re.get_handler(duel).is_original_code_rule(duel, e.get_label(duel) as u32)),
    );
    e1.set_reset(duel, RESET_PHASE | PHASE_END);
    duel.register_effect(e1, tp);
    let e2 = e1.clone_effect(duel);
    e2.set_target_range(duel, 1, 0);
    e2.set_reset(duel, 0);
    duel.register_effect(e2, tp);
    Ok(())
}
