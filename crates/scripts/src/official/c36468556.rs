// Ceasefire
use crate::prelude::*;
use crate::testing::{HAND, MZONE, OPPONENT, SZONE};

const ID: u32 = 36468556;

// In the hand and Set, with Sangans on both sides.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, SZONE),
    (26202165, MZONE),
    (26202165, MZONE),
    (26202165, MZONE | OPPONENT),
    (26202165, MZONE | OPPONENT),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_category(duel, CATEGORY_POSITION + CATEGORY_DAMAGE);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_target(duel, target);
    e1.set_operation(duel, activate);
    c.register_effect(duel, e1);
}

fn tgfilter(c: CardId, duel: &Duel) -> bool {
    (c.is_faceup(duel) && c.is_type(duel, TYPE_EFFECT)) || c.is_position(duel, POS_FACEDOWN_DEFENSE)
}

fn filter(c: CardId, duel: &Duel) -> bool {
    c.is_faceup(duel) && c.is_type(duel, TYPE_EFFECT)
}

fn target(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(tgfilter, tp, LOCATION_MZONE, LOCATION_MZONE, 1, None));
    }
    let dam = duel.get_matching_group_count(filter, tp, LOCATION_MZONE, LOCATION_MZONE, None) as u32 * 500;
    duel.set_target_player(1 - tp);
    duel.set_target_param(dam);
    if dam > 0 {
        duel.set_operation_info(0, CATEGORY_DAMAGE, None, 0, 1 - tp, dam as i32);
    }
    Ok(false)
}

fn activate(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    let g = duel.get_matching_group(
        |c, duel| c.is_position(duel, POS_FACEDOWN_DEFENSE),
        tp,
        LOCATION_MZONE,
        LOCATION_MZONE,
        None,
    );
    duel.change_position_full(g, [0, 0, 0, POS_FACEUP_DEFENSE], true)?;
    let p: u8 = duel.get_chain_info(0, CHAININFO_TARGET_PLAYER);
    let dam = duel.get_matching_group_count(filter, tp, LOCATION_MZONE, LOCATION_MZONE, None) as u32 * 500;
    duel.damage(p, dam, REASON_EFFECT)?;
    Ok(())
}
