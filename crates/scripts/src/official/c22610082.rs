// The Mask of Remnants
use crate::prelude::*;
use crate::testing::SZONE;

const ID: u32 = 22610082;

pub const SUPPORT: &[(u32, u16)] = &[(ID, SZONE)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_category(duel, CATEGORY_TODECK);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_cost(duel, cost);
    e1.set_target(duel, target);
    e1.set_operation(duel, activate);
    c.register_effect(duel, e1);
}

fn cost(duel: &mut Duel, a: &Args, _: i32) -> Script<bool> {
    a.e.set_label(duel, 1);
    Ok(true)
}

fn target(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let e = a.e;
    let c = e.get_handler(duel);
    if chk == 0 {
        if e.get_label(duel) == 0 {
            return Ok(false);
        }
        e.set_label(duel, 0);
        return Ok(c.is_able_to_deck(duel));
    }
    e.set_label(duel, 0);
    duel.set_operation_info(0, CATEGORY_TODECK, Some(c.into()), 1, 0, 0);
    Ok(false)
}

fn activate(duel: &mut Duel, a: &Args) -> Script<()> {
    let c = a.e.get_handler(duel);
    if c.is_relate_to_effect(duel, a.e) {
        c.cancel_to_grave(duel);
        duel.send_to_deck(c, PLAYER_NONE, SEQ_DECKSHUFFLE, REASON_EFFECT)?;
    }
    Ok(())
}
