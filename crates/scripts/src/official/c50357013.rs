// Tokusano Shinkyojin
use crate::prelude::*;
use crate::testing::{Check, HAND};

const ID: u32 = 50357013;

// In the hand with two Level 5 Normal Monsters (Shapesnatch) to send for the cost.
pub const SUPPORT: &[(u32, u16)] = &[(ID, HAND), (4035199, HAND), (4035199, HAND)];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(0)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_category(duel, CATEGORY_DRAW);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_property(duel, EFFECT_FLAG_PLAYER_TARGET);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_cost(duel, cost);
    e1.set_target(duel, stock::draw_target::<2>);
    e1.set_operation(duel, stock::draw);
    c.register_effect(duel, e1);
}

fn cfilter(c: CardId, duel: &Duel) -> bool {
    c.has_level(duel) && c.is_able_to_grave_as_cost(duel) && (c.is_faceup(duel) || c.is_location(duel, LOCATION_HAND))
}

fn cost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let tp = a.tp;
    let g = duel.get_matching_group(cfilter, tp, LOCATION_HAND | LOCATION_MZONE, 0, None);
    let ct = g.len();
    if chk == 0 {
        return Ok(ct > 0 && g.check_with_sum_equal(duel, CardId::get_level, 10, 1, ct));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TOGRAVE);
    let cg = g.select_with_sum_equal(duel, tp, CardId::get_level, 10, 1, ct as i32)?;
    duel.send_to_grave(cg, REASON_COST)?;
    Ok(false)
}
