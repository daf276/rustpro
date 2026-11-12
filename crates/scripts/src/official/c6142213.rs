// Turbo Rocket
use crate::prelude::*;
use crate::testing::{MZONE, OPPONENT};

const ID: u32 = 6142213;

// Copies on the field to attack the opponent's Sangans.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (ID, MZONE),
    (ID, MZONE),
    (26202165, MZONE | OPPONENT),
    (26202165, MZONE | OPPONENT),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_DAMAGE);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_F);
    e1.set_code(duel, EVENT_BATTLED);
    e1.set_condition(duel, condition);
    e1.set_target(duel, target);
    e1.set_operation(duel, operation);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_SINGLE);
    e2.set_code(duel, EFFECT_INDESTRUCTABLE_BATTLE);
    e2.set_condition(duel, indcon);
    e2.set_value(duel, 1);
    c.register_effect(duel, e2);
}

fn condition(duel: &Duel, a: &Args) -> bool {
    duel.get_attacker() == Some(a.e.get_handler(duel)) && duel.get_attack_target().is_some()
}

// Lua halves with a float, which the core rounds.
fn half_attack(duel: &Duel) -> i32 {
    let target = duel.get_attack_target();
    unported!(duel, target.is_none());
    target.map_or(0, |tc| (f64::from(tc.get_attack(duel)) / 2.0).round() as i32)
}

fn target(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(true);
    }
    let dam = half_attack(duel);
    duel.set_target_player(1 - tp);
    duel.set_target_param(dam as u32);
    duel.set_operation_info(0, CATEGORY_DAMAGE, None, 0, 1 - tp, dam);
    Ok(false)
}

fn operation(duel: &mut Duel, _: &Args) -> Script<()> {
    let p: u8 = duel.get_chain_info(0, CHAININFO_TARGET_PLAYER);
    let dam = half_attack(duel);
    duel.damage(p, dam as u32, REASON_EFFECT)?;
    Ok(())
}

fn indcon(duel: &Duel, a: &Args) -> bool {
    duel.get_attacker() == Some(a.e.get_handler(duel))
}
