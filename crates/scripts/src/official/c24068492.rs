// Just Desserts
use crate::prelude::*;
use crate::testing::{HAND, MZONE};

const ID: u32 = 24068492;

// In the hand to Activate, with monsters on the opponent's field and its own.
pub const SUPPORT: &[(u32, u16)] = &[(ID, HAND), (70781052, MZONE), (14851496, MZONE)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_category(duel, CATEGORY_DAMAGE);
    e1.set_property(duel, EFFECT_FLAG_PLAYER_TARGET);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_hint_timing(duel, 0, TIMINGS_CHECK_MONSTER | TIMING_DRAW_PHASE);
    e1.set_target(duel, target);
    e1.set_operation(duel, activate);
    c.register_effect(duel, e1);
}

fn target(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(aux::TRUE, tp, 0, LOCATION_MZONE, 1, None));
    }
    duel.set_target_player(1 - tp);
    let dam = duel.get_field_group_count(1 - tp, LOCATION_MZONE, 0) as u32 * 500;
    duel.set_target_param(dam);
    duel.set_operation_info(0, CATEGORY_DAMAGE, None, 0, 1 - tp, dam as i32);
    Ok(false)
}

fn activate(duel: &mut Duel, a: &Args) -> Script<()> {
    let p: u8 = duel.get_chain_info(0, CHAININFO_TARGET_PLAYER);
    let dam = duel.get_field_group_count(1 - a.tp, LOCATION_MZONE, 0) as u32 * 500;
    duel.damage(p, dam, REASON_EFFECT)?;
    Ok(())
}
