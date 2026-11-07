// Chaos Greed
use crate::prelude::*;
use crate::testing::{HAND, REMOVED};

const ID: u32 = 97439308;

// Four banished cards in the hand with the Graveyard still empty.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (26202165, REMOVED),
    (26202165, REMOVED),
    (26202165, REMOVED),
    (26202165, REMOVED),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_category(duel, CATEGORY_DRAW);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_property(duel, EFFECT_FLAG_PLAYER_TARGET);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_condition(duel, condition);
    e1.set_target(duel, target);
    e1.set_operation(duel, activate);
    c.register_effect(duel, e1);
}

fn condition(duel: &Duel, a: &Args) -> bool {
    duel.get_field_group_count(a.tp, LOCATION_REMOVED, 0) >= 4 && duel.get_field_group_count(a.tp, LOCATION_GRAVE, 0) == 0
}

fn target(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.is_player_can_draw(tp, 2));
    }
    duel.set_target_player(tp);
    duel.set_target_param(2);
    duel.set_operation_info(0, CATEGORY_DRAW, None, 0, tp, 2);
    Ok(false)
}

fn activate(duel: &mut Duel, _: &Args) -> Script<()> {
    let p: u8 = duel.get_chain_info(0, CHAININFO_TARGET_PLAYER);
    let d: u32 = duel.get_chain_info(0, CHAININFO_TARGET_PARAM);
    duel.draw(p, d, REASON_EFFECT)?;
    Ok(())
}
