// Des Kangaroo
use crate::prelude::*;
use crate::testing::{Check, HAND, MZONE, OPPONENT};

const ID: u32 = 78613627;

// On the field in Defense Position (forced by Level Limit - Area B) against the opponent's Sangans, which attack with less ATK than its DEF.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (ID, MZONE),
    (3136426, HAND),
    (3136426, HAND),
    (26202165, MZONE | OPPONENT),
    (26202165, MZONE | OPPONENT),
    (26202165, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_DESTROY);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_F);
    e1.set_code(duel, EVENT_DAMAGE_STEP_END);
    e1.set_condition(duel, condition);
    e1.set_target(duel, target);
    e1.set_operation(duel, operation);
    c.register_effect(duel, e1);
}

fn condition(duel: &Duel, a: &Args) -> bool {
    let c = a.e.get_handler(duel);
    let attacker = duel.get_attacker();
    unported!(duel, attacker.is_none());
    duel.get_attack_target() == Some(c)
        && c.get_battle_position(duel) & POS_DEFENSE != 0
        && attacker.is_some_and(|attacker| attacker.get_attack(duel) < c.get_defense(duel))
}

fn target(duel: &mut Duel, _: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(true);
    }
    let attacker = duel.get_attacker();
    duel.set_operation_info(0, CATEGORY_DESTROY, attacker.map(Into::into), 1, 0, 0);
    Ok(false)
}

fn operation(duel: &mut Duel, _: &Args) -> Script<()> {
    if let Some(attacker) = duel.get_attacker()
        && attacker.is_relate_to_battle(duel)
    {
        duel.destroy(attacker, REASON_EFFECT)?;
    }
    Ok(())
}
