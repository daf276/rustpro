// Sword Hunter
use engine::constants::MSG_EQUIP;

use crate::prelude::*;
use crate::testing::{Check, MZONE, OPPONENT};
use crate::utility::equip_by_effect_limit;

const ID: u32 = 51345461;

// On the field with copies of the opponent's to destroy by battle and be equipped (MSG_EQUIP names the equipped card).
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (ID, MZONE),
    (ID, MZONE),
    (ID, MZONE),
    (ID, MZONE | OPPONENT),
    (ID, MZONE | OPPONENT),
    (ID, MZONE | OPPONENT),
    (ID, MZONE | OPPONENT),
];

pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0)), Check::Message(MSG_EQUIP)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_EQUIP);
    e1.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_F);
    e1.set_code(duel, EVENT_PHASE | PHASE_BATTLE);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_count_limit(duel, 1, 0);
    e1.set_target(duel, eqtg);
    e1.set_operation(duel, eqop);
    c.register_effect(duel, e1);
    aux::add_er_equip_limit(duel, c, None, e1);
}

fn filter(c: CardId, duel: &Duel, rc: CardId, tid: i32) -> bool {
    c.is_reason(duel, REASON_BATTLE) && c.get_reason_card(duel) == Some(rc) && c.get_turn_id(duel) == tid && !c.is_forbidden(duel)
}

fn eqtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(true);
    }
    let (rc, tid) = (a.e.get_handler(duel), duel.get_turn_count());
    let g = duel.get_matching_group(|c, duel| filter(c, duel, rc, tid), a.tp, LOCATION_GRAVE, LOCATION_GRAVE, None);
    duel.set_operation_info(0, CATEGORY_EQUIP, Some(g), g.len() as u8, 0, 0);
    Ok(false)
}

fn equipop(duel: &mut Duel, c: CardId, e: EffectId, tp: u8, tc: CardId, chk: bool) -> Script<()> {
    duel.equip_full(tp, tc, c, false, chk)?;
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_property(duel, EFFECT_FLAG_OWNER_RELATE);
    e1.set_code(duel, EFFECT_EQUIP_LIMIT);
    e1.set_reset(duel, RESET_EVENT | RESETS_STANDARD);
    e1.set_value(duel, Value::Card(|duel, e, c| equip_by_effect_limit(duel, e, c).into()));
    e1.set_label_object(duel, e.get_label_object(duel));
    tc.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_EQUIP);
    e2.set_property(duel, EFFECT_FLAG_OWNER_RELATE + EFFECT_FLAG_IGNORE_IMMUNE);
    e2.set_code(duel, EFFECT_UPDATE_ATTACK);
    e2.set_reset(duel, RESET_EVENT | RESETS_STANDARD);
    e2.set_value(duel, 200);
    tc.register_effect(duel, e2);
    Ok(())
}

fn eqop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let ft = duel.get_location_count(tp, LOCATION_SZONE);
    if ft <= 0 {
        return Ok(());
    }
    let c = e.get_handler(duel);
    if !c.is_relate_to_effect(duel, e) || c.is_facedown(duel) {
        return Ok(());
    }
    let tid = duel.get_turn_count();
    let g = duel.get_matching_group(|tc, duel| filter(tc, duel, c, tid), tp, LOCATION_GRAVE, LOCATION_GRAVE, None);
    if g.is_empty() || g.len() as i32 > ft {
        return Ok(());
    }
    for tc in g {
        equipop(duel, c, e, tp, tc, true)?;
    }
    duel.equip_complete()?;
    Ok(())
}
