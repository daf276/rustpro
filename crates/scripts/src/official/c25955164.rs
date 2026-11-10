// Sanga of the Thunder
use crate::prelude::*;
use crate::testing::{Check, MZONE, OPPONENT};

const ID: u32 = 25955164;

// Copies on the field to be attacked by the opponent's Sangan and Elfnote Lucina.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (ID, MZONE),
    (ID, MZONE),
    (26202165, MZONE | OPPONENT),
    (13597785, MZONE | OPPONENT),
    (13597785, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    add_atkdown(duel, c, ID);
}

// The effect Sanga of the Thunder, Kazejin and Suijin share, with the code of the card that has it.
pub fn add_atkdown(duel: &mut Duel, c: CardId, id: u32) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(id, 0));
    e1.set_category(duel, CATEGORY_ATKCHANGE);
    e1.set_type(duel, EFFECT_TYPE_QUICK_O);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_code(duel, EVENT_PRE_DAMAGE_CALCULATE);
    e1.set_property(duel, EFFECT_FLAG_NO_TURN_RESET);
    e1.set_count_limit(duel, 1, 0);
    e1.set_condition(duel, condition);
    e1.set_target(duel, target);
    e1.set_operation(duel, operation);
    c.register_effect(duel, e1);
}

fn condition(duel: &Duel, a: &Args) -> bool {
    duel.is_turn_player(1 - a.tp) && duel.get_attack_target() == Some(a.e.get_handler(duel))
}

fn target(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let attacker = duel.get_attacker();
    if chk == 0 {
        return Ok(attacker.is_some_and(|attacker| attacker.is_can_be_effect_target(duel, a.e)));
    }
    if let Some(attacker) = attacker {
        duel.set_target_card(attacker);
    }
    Ok(false)
}

fn operation(duel: &mut Duel, a: &Args) -> Script<()> {
    if let Some(tc) = duel.get_first_target()
        && tc.is_relate_to_effect(duel, a.e)
    {
        let e1 = duel.create_effect(a.e.get_handler(duel));
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_code(duel, EFFECT_SET_ATTACK_FINAL);
        e1.set_reset(duel, RESET_PHASE | PHASE_DAMAGE_CAL);
        e1.set_value(duel, 0);
        tc.register_effect(duel, e1);
    }
    Ok(())
}
