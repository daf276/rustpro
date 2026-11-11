// Pot of Desires
use crate::prelude::*;
use crate::testing::HAND;

const ID: u32 = 35261759;

pub const SUPPORT: &[(u32, u16)] = &[(ID, HAND)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_category(duel, CATEGORY_DRAW);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_property(duel, EFFECT_FLAG_PLAYER_TARGET);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_count_limit_full(duel, 1, ID, EFFECT_COUNT_CODE_OATH);
    e1.set_cost(duel, cost);
    e1.set_target(duel, stock::draw_target::<2>);
    e1.set_operation(duel, stock::draw);
    c.register_effect(duel, e1);
}

fn cost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let tp = a.tp;
    let g = duel.get_decktop_group(tp, 10);
    if chk == 0 {
        return Ok(
            g.filter_count(duel, |c, duel| c.is_able_to_remove_as_cost_full(duel, POS_FACEDOWN), None) == 10
                && duel.get_field_group_count(tp, LOCATION_DECK, 0) >= 12,
        );
    }
    duel.disable_shuffle_check(true);
    duel.remove(g, POS_FACEDOWN, REASON_COST)?;
    Ok(false)
}
