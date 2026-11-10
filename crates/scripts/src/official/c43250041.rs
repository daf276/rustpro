// Draining Shield
use crate::prelude::*;
use crate::testing::{Check, HAND, MZONE, OPPONENT, SZONE};

const ID: u32 = 43250041;

// In the hand and Set, against the opponent's Terra the Terribles attacking.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, SZONE),
    (63308047, MZONE | OPPONENT),
    (63308047, MZONE | OPPONENT),
    (63308047, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(0)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_category(duel, CATEGORY_RECOVER);
    e1.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_ATTACK_ANNOUNCE);
    e1.set_condition(duel, |duel, a| duel.is_turn_player(1 - a.tp));
    e1.set_target(duel, target);
    e1.set_operation(duel, activate);
    c.register_effect(duel, e1);
}

fn target(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tg = duel.get_attacker();
    if let Some(chkc) = chkc {
        return Ok(Some(chkc) == tg);
    }
    let Some(tg) = tg else { return Ok(false) };
    if chk == 0 {
        return Ok(tg.is_on_field(duel) && tg.is_can_be_effect_target(duel, a.e));
    }
    duel.set_target_card(tg);
    let rec = tg.get_attack(duel);
    duel.set_operation_info(0, CATEGORY_RECOVER, None, 0, a.tp, rec);
    Ok(false)
}

fn activate(duel: &mut Duel, a: &Args) -> Script<()> {
    let Some(tc) = duel.get_first_target() else { return Ok(()) };
    if tc.is_relate_to_effect(duel, a.e) && tc.is_faceup(duel) && tc.can_attack(duel) && duel.negate_attack()? {
        let atk = tc.get_attack(duel) as u32;
        duel.recover(a.tp, atk, REASON_EFFECT)?;
    }
    Ok(())
}
