// Marshmallon
use crate::prelude::*;
use crate::testing::{MZONE, OPPONENT};

const ID: u32 = 31305911;

// Copies on the field to be attacked face-down by the opponent's Sangan and Elfnote Lucina.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (ID, MZONE),
    (ID, MZONE),
    (26202165, MZONE | OPPONENT),
    (13597785, MZONE | OPPONENT),
    (13597785, MZONE | OPPONENT),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_DAMAGE);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_F);
    e1.set_code(duel, EVENT_BATTLED);
    e1.set_condition(duel, condition);
    e1.set_target(duel, target);
    e1.set_operation(duel, stock::damage);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_SINGLE);
    e2.set_code(duel, EFFECT_INDESTRUCTABLE_BATTLE);
    e2.set_value(duel, 1);
    c.register_effect(duel, e2);
}

fn condition(duel: &Duel, a: &Args) -> bool {
    let c = a.e.get_handler(duel);
    let pos = c.get_battle_position(duel);
    duel.get_attack_target() == Some(c)
        && (pos == POS_FACEDOWN_DEFENSE || pos == POS_FACEDOWN_ATTACK)
        && c.is_location(duel, LOCATION_MZONE)
}

fn target(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(true);
    }
    let attacker = duel.get_attacker();
    unported!(duel, attacker.is_none());
    if let Some(attacker) = attacker {
        let controler = attacker.get_controler(duel);
        duel.set_target_player(controler);
    }
    duel.set_target_param(1000);
    duel.set_operation_info(0, CATEGORY_DAMAGE, None, 0, 1 - a.tp, 1000);
    Ok(false)
}
