// Magic Cylinder
use crate::prelude::*;
use crate::testing::{MZONE, OPPONENT, SZONE};

const ID: u32 = 62279055;

// Set, against attacking monsters of the opponent.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, SZONE),
    (50930991, MZONE | OPPONENT),
    (50930991, MZONE | OPPONENT),
    (50930991, MZONE | OPPONENT),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_category(duel, CATEGORY_DAMAGE);
    e1.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_ATTACK_ANNOUNCE);
    e1.set_condition(duel, |duel, a| a.tp == 1 - duel.get_turn_player());
    e1.set_target(duel, target);
    e1.set_operation(duel, activate);
    c.register_effect(duel, e1);
}

fn target(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tg = duel.get_attacker();
    if let Some(chkc) = chkc {
        return Ok(Some(chkc) == tg);
    }
    if chk == 0 {
        return Ok(tg.is_some_and(|tg| tg.is_on_field(duel) && tg.is_can_be_effect_target(duel, a.e)));
    }
    let Some(tg) = tg else { return Ok(false) };
    duel.set_target_card(tg);
    let dam = tg.get_attack(duel);
    duel.set_operation_info(0, CATEGORY_DAMAGE, None, 0, 1 - a.tp, dam);
    Ok(false)
}

fn activate(duel: &mut Duel, a: &Args) -> Script<()> {
    if let Some(tc) = duel.get_first_target()
        && tc.is_relate_to_effect(duel, a.e)
        && duel.negate_attack()?
    {
        duel.damage(1 - a.tp, tc.get_attack(duel).max(0) as u32, REASON_EFFECT)?;
    }
    Ok(())
}
