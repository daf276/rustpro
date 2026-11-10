// Cooky☆Yummy
use super::c4215180::{add_special_summon_procedure, is_summoned_by_synchro_effect};
use crate::prelude::*;
use crate::testing::{Check, GRAVE, HAND, MZONE, OPPONENT};

const ID: u32 = 68810435;

// In the hand and the Graveyard, with Cooky★Yummy Way (a Level 2 Synchro) on the field to Special Summon it from the hand and
// to Special Summon it from the Graveyard, and the opponent's Raigeki to respond to and monsters to target.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, HAND),
    (ID, GRAVE),
    (ID, GRAVE),
    (67098897, MZONE),
    (67098897, MZONE),
    (74677422, MZONE | OPPONENT),
    (12580477, HAND | OPPONENT),
];
pub const CHECKS: &[Check] = &[Check::SpecialSummoned, Check::ResolvesEffect(aux::stringid(ID, 1))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    add_special_summon_procedure::<ID>(duel, c);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_ATKCHANGE);
    e2.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e2.set_property(duel, EFFECT_FLAG_DELAY + EFFECT_FLAG_CARD_TARGET);
    e2.set_code(duel, EVENT_SUMMON_SUCCESS);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_target(duel, atktg);
    e2.set_operation(duel, atkop);
    c.register_effect(duel, e2);
    let e3 = e2.clone_effect(duel);
    e3.set_category(duel, CATEGORY_ATKCHANGE + CATEGORY_DESTROY);
    e3.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    c.register_effect(duel, e3);
}

fn atktg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let sp_chk = is_summoned_by_synchro_effect(duel, a);
    if let Some(chkc) = chkc {
        return Ok(chkc.is_controler(duel, 1 - tp) && chkc.is_location(duel, LOCATION_MZONE) && chkc.is_faceup(duel));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(CardId::is_faceup, tp, 0, LOCATION_MZONE, 1, None));
    }
    e.set_label(duel, i64::from(sp_chk));
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TARGET);
    let g = duel.select_target(tp, CardId::is_faceup, tp, 0, LOCATION_MZONE, 1, 1, None)?;
    if sp_chk {
        duel.set_possible_operation_info(0, CATEGORY_ATKCHANGE, g, 1, tp, -1000);
        duel.set_possible_operation_info(0, CATEGORY_DESTROY, g, 1, tp, 0);
    } else {
        duel.set_operation_info(0, CATEGORY_ATKCHANGE, g, 1, tp, -1000);
    }
    Ok(false)
}

fn atkop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let Some(tc) = duel.get_first_target().filter(|tc| tc.is_relate_to_effect(duel, e)) else {
        return Ok(());
    };
    let sp_chk = e.get_label(duel) == 1;
    let (b1, b2) = (tc.is_faceup(duel), sp_chk);
    if !(b1 || b2) {
        return Ok(());
    }
    let op = if sp_chk {
        duel.select_effect(tp, &[(b1, aux::stringid(ID, 2)), (b2, aux::stringid(ID, 3))])?
    } else {
        Some(1)
    };
    if op == Some(1) {
        tc.update_attack(duel, -1000, None, Some(e.get_handler(duel)), None);
    } else if op == Some(2) {
        duel.destroy(tc, REASON_EFFECT)?;
    }
    Ok(())
}
