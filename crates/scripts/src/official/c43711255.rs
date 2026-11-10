// Prohibition
use crate::prelude::*;
use crate::testing::{Check, HAND, SZONE};

const ID: u32 = 43711255;

pub const SUPPORT: &[(u32, u16)] = &[(ID, HAND), (ID, SZONE), (ID, HAND)];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(0)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_target(duel, target);
    e1.set_operation(duel, activate);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_FIELD);
    e2.set_property(duel, EFFECT_FLAG_SET_AVAILABLE);
    e2.set_code(duel, EFFECT_FORBIDDEN);
    e2.set_range(duel, LOCATION_SZONE);
    e2.set_target_range(duel, 0x7f, 0x7f);
    e2.set_target_card(duel, bantg);
    e2.set_label_object(duel, Some(e1));
    c.register_effect(duel, e2);
}

fn bantg(duel: &Duel, e: EffectId, c: CardId) -> bool {
    let label = e.get_label_object(duel).map_or(0, |e1| e1.get_label(duel));
    c.is_original_code_rule(duel, label as u32) && (!c.is_on_field(duel) || c.get_real_field_id(duel) > e.get_field_id(duel))
}

fn target(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if chk == 0 {
        return Ok(true);
    }
    e.set_label(duel, 0);
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_CODE);
    let ac = duel.announce_card(tp, &[])?;
    duel.set_target_param(ac);
    duel.set_operation_info(0, CATEGORY_ANNOUNCE, None, 0, tp, ANNOUNCE_CARD as i32);
    Ok(false)
}

fn activate(duel: &mut Duel, a: &Args) -> Script<()> {
    let ac: u32 = duel.get_chain_info(0, CHAININFO_TARGET_PARAM);
    a.e.set_label(duel, i64::from(ac));
    let c = a.e.get_handler(duel);
    c.set_hint(duel, CHINT_CARD, u64::from(ac));
    duel.adjust_instantly(Some(c));
    Ok(())
}
