// Spear Dragon
use crate::prelude::*;
use crate::testing::{Check, MZONE, OPPONENT};
use engine::constants::MSG_POS_CHANGE;

const ID: u32 = 31553716;

// On the field in Attack Position, with Sangan for it to attack.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (ID, MZONE),
    (ID, MZONE),
    (26202165, MZONE | OPPONENT),
    (26202165, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[Check::Message(MSG_POS_CHANGE)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_CONTINUOUS);
    e1.set_code(duel, EVENT_DAMAGE_STEP_END);
    e1.set_condition(duel, poscon);
    e1.set_operation(duel, posop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_SINGLE);
    e2.set_code(duel, EFFECT_PIERCE);
    c.register_effect(duel, e2);
}

fn poscon(duel: &Duel, a: &Args) -> bool {
    let c = a.e.get_handler(duel);
    duel.get_attacker() == Some(c) && c.is_relate_to_battle(duel)
}

fn posop(duel: &mut Duel, a: &Args) -> Script<()> {
    let c = a.e.get_handler(duel);
    if c.is_attack_pos(duel) {
        duel.change_position(c, POS_FACEUP_DEFENSE)?;
    }
    Ok(())
}
