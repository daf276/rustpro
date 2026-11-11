// Sacred Sword of Seven Stars
use crate::prelude::*;
use crate::testing::{HAND, MZONE};

const ID: u32 = 45725480;

// In the hand, with Level 7 monsters (Red-Eyes Black Dragon) in the hand and on the field to banish.
pub const SUPPORT: &[(u32, u16)] = &[(ID, HAND), (74677422, HAND), (74677422, MZONE)];

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

fn filter(c: CardId, duel: &Duel) -> bool {
    (c.is_location(duel, LOCATION_HAND) || c.is_faceup(duel)) && c.get_level(duel) == 7 && c.is_able_to_remove_as_cost(duel)
}

fn cost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    stock::banish_cost(duel, a, chk, filter, LOCATION_HAND | LOCATION_MZONE)
}
