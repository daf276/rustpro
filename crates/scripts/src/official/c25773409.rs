// Legendary Jujitsu Master
use crate::prelude::*;
use crate::testing::{Check, MZONE, OPPONENT};

const ID: u32 = 25773409;

// Copies on the field, and the opponent's attackers to hit them.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (ID, MZONE),
    (ID, MZONE),
    (13597785, MZONE | OPPONENT),
    (13597785, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_TODECK);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_F);
    e1.set_code(duel, EVENT_DAMAGE_STEP_END);
    e1.set_condition(duel, condition);
    e1.set_target(duel, target);
    e1.set_operation(duel, operation);
    c.register_effect(duel, e1);
}

fn condition(duel: &Duel, a: &Args) -> bool {
    let c = a.e.get_handler(duel);
    duel.get_attack_target() == Some(c) && c.get_battle_position(duel) & POS_DEFENSE != 0
}

fn target(duel: &mut Duel, _: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(true);
    }
    let attacker = duel.get_attacker();
    duel.set_operation_info(0, CATEGORY_TODECK, attacker.map(Into::into), 1, 0, 0);
    Ok(false)
}

fn operation(duel: &mut Duel, _: &Args) -> Script<()> {
    let Some(a) = duel.get_attacker() else { return Ok(()) };
    if !a.is_relate_to_battle(duel) {
        return Ok(());
    }
    duel.send_to_deck(a, PLAYER_NONE, SEQ_DECKTOP, REASON_EFFECT)?;
    Ok(())
}
