// Broww, Huntsman of Dark World
use crate::prelude::*;
use crate::testing::{Check, HAND, OPPONENT};

const ID: u32 = 79126789;

// In the hand, to be discarded by Dark World Dealings from either side.
pub const SUPPORT: &[(u32, u16)] = &[(ID, HAND), (74117290, HAND), (74117290, HAND | OPPONENT)];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_DRAW);
    e1.set_property(duel, EFFECT_FLAG_PLAYER_TARGET);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_F);
    e1.set_code(duel, EVENT_TO_GRAVE);
    e1.set_condition_mut(duel, super::c34230233::descon);
    e1.set_target(duel, drtg);
    e1.set_operation(duel, stock::draw);
    c.register_effect(duel, e1);
}

fn drtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if chk == 0 {
        return Ok(true);
    }
    duel.set_target_player(tp);
    let count = if tp == a.rp || i64::from(tp) != e.get_label(duel) { 1 } else { 2 };
    duel.set_target_param(count);
    duel.set_operation_info(0, CATEGORY_DRAW, None, 0, tp, count as i32);
    Ok(false)
}
