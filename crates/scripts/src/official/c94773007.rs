// Jirai Gumo
use crate::prelude::*;
use crate::testing::{MZONE, OPPONENT};

const ID: u32 = 94773007;

pub const TOSS_COIN: bool = true;

// Attacks into the opponent's Sangan.
pub const SUPPORT: &[(u32, u16)] = &[(ID, MZONE), (ID, MZONE), (26202165, MZONE | OPPONENT), (26202165, MZONE | OPPONENT)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_COIN);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_F);
    e1.set_code(duel, EVENT_ATTACK_ANNOUNCE);
    e1.set_target(duel, attg);
    e1.set_operation(duel, atop);
    c.register_effect(duel, e1);
}

fn attg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(true);
    }
    duel.set_operation_info(0, CATEGORY_COIN, None, 0, tp, 1);
    Ok(false)
}

fn atop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    if !duel.call_coin(tp)? {
        duel.set_lp(tp, (duel.get_lp(tp) + 1) / 2);
    }
    Ok(())
}
