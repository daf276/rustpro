// Dimensional Prison
use crate::prelude::*;

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_category(duel, CATEGORY_REMOVE);
    e1.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_ATTACK_ANNOUNCE);
    e1.set_condition(duel, condition);
    e1.set_target(duel, target);
    e1.set_operation(duel, activate);
    c.register_effect(duel, e1);
}

fn condition(duel: &Duel, a: &Args) -> bool {
    duel.is_turn_player(1 - a.tp)
}

fn target(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let Some(tg) = duel.get_attacker() else { return Ok(false) };
    if let Some(chkc) = chkc {
        return Ok(chkc == tg);
    }
    if chk == 0 {
        return Ok(tg.is_on_field(duel) && tg.is_able_to_remove(duel) && tg.is_can_be_effect_target(duel, a.e));
    }
    duel.set_target_card(tg);
    duel.set_operation_info(0, CATEGORY_REMOVE, Some(tg.into()), 1, 0, 0);
    Ok(false)
}

fn activate(duel: &mut Duel, a: &Args) -> Script<()> {
    let Some(tc) = duel.get_first_target() else { return Ok(()) };
    if tc.is_relate_to_effect(duel, a.e) && tc.can_attack(duel) && !tc.is_status(duel, STATUS_ATTACK_CANCELED) {
        duel.remove(tc, POS_FACEUP, REASON_EFFECT)?;
    }
    Ok(())
}
