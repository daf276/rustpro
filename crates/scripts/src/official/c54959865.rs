// Neo-Spacian Air Hummingbird
use crate::prelude::*;
use crate::testing::{Check, MZONE};

const ID: u32 = 54959865;

// On the field, to recover from the cards in the opponent's hand.
pub const SUPPORT: &[(u32, u16)] = &[(ID, MZONE)];
pub const CHECKS: &[Check] = &[Check::Resolves];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_property(duel, EFFECT_FLAG_PLAYER_TARGET);
    e1.set_category(duel, CATEGORY_RECOVER);
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_count_limit(duel, 1, 0);
    e1.set_target(duel, target);
    e1.set_operation(duel, operation);
    c.register_effect(duel, e1);
}

fn target(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.get_field_group_count(tp, 0, LOCATION_HAND) > 0);
    }
    duel.set_target_player(tp);
    duel.set_operation_info(0, CATEGORY_RECOVER, None, 0, tp, 0);
    Ok(false)
}

fn operation(duel: &mut Duel, a: &Args) -> Script<()> {
    let rt = duel.get_field_group_count(a.tp, 0, LOCATION_HAND) as u32 * 500;
    let p: u8 = duel.get_chain_info(0, CHAININFO_TARGET_PLAYER);
    duel.recover(p, rt, REASON_EFFECT)?;
    Ok(())
}
