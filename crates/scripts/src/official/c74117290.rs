// Dark World Dealings
use crate::prelude::*;
use crate::testing::{Check, HAND};

const ID: u32 = 74117290;

pub const SUPPORT: &[(u32, u16)] = &[(ID, HAND)];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(0)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_category(duel, CATEGORY_HANDES + CATEGORY_DRAW);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_target(duel, target);
    e1.set_operation(duel, activate);
    c.register_effect(duel, e1);
}

fn target(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.is_player_can_draw(tp, 1) && duel.is_player_can_draw(1 - tp, 1));
    }
    duel.set_operation_info(0, CATEGORY_HANDES, None, 0, PLAYER_ALL, 1);
    duel.set_operation_info(0, CATEGORY_DRAW, None, 0, PLAYER_ALL, 1);
    Ok(false)
}

fn activate(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    let h1 = duel.draw(tp, 1, REASON_EFFECT)?;
    let h2 = duel.draw(1 - tp, 1, REASON_EFFECT)?;
    if h1 > 0 || h2 > 0 {
        duel.break_effect()?;
    }
    for (drawn, player) in [(h1, tp), (h2, 1 - tp)] {
        if drawn > 0 {
            duel.shuffle_hand(player)?;
            duel.discard_hand(player, aux::TRUE, 1, 1, REASON_EFFECT | REASON_DISCARD)?;
        }
    }
    Ok(())
}
