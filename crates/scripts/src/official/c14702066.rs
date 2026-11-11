// Cannon Soldier MK-2
use crate::prelude::*;
use crate::testing::MZONE;

const ID: u32 = 14702066;

// Sangan and Ojama Green on the field to Tribute.
pub const SUPPORT: &[(u32, u16)] = &[(ID, MZONE), (26202165, MZONE), (12482652, MZONE)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_DAMAGE);
    e1.set_property(duel, EFFECT_FLAG_PLAYER_TARGET);
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_cost(duel, cost);
    e1.set_target(duel, target);
    e1.set_operation(duel, stock::damage);
    c.register_effect(duel, e1);
}

fn cost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.check_release_group_cost(tp, aux::TRUE, (2, 2), false, None, None));
    }
    let sg = duel.select_release_group_cost(tp, aux::TRUE, (2, 2), false, None, None)?;
    duel.release(sg, REASON_COST)?;
    Ok(false)
}

fn target(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(true);
    }
    duel.set_target_player(1 - tp);
    duel.set_target_param(1500);
    duel.set_operation_info(0, CATEGORY_DAMAGE, None, 0, 1 - tp, 1500);
    Ok(false)
}
