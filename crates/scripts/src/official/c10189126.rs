// Masked Sorcerer
use crate::prelude::*;
use crate::testing::{MZONE, OPPONENT};

const ID: u32 = 10189126;

// Two on the field, and weak monsters (Psychic Kappa) of the opponent's to deal battle damage to.
pub const SUPPORT: &[(u32, u16)] = &[(ID, MZONE), (ID, MZONE), (7892180, MZONE | OPPONENT), (7892180, MZONE | OPPONENT)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_DRAW);
    e1.set_property(duel, EFFECT_FLAG_PLAYER_TARGET);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_F);
    e1.set_code(duel, EVENT_BATTLE_DAMAGE);
    e1.set_condition(duel, |_, a| a.ep != a.tp);
    e1.set_target(duel, target);
    e1.set_operation(duel, stock::draw);
    c.register_effect(duel, e1);
}

fn target(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(true);
    }
    duel.set_target_player(tp);
    duel.set_target_param(1);
    duel.set_operation_info(0, CATEGORY_DRAW, None, 0, tp, 1);
    Ok(false)
}
