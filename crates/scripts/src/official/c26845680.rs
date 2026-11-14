// Orcustrated Return
use crate::prelude::*;
use crate::testing::{Check, HAND, MZONE};

const ID: u32 = 26845680;

pub const LISTED_SERIES: &[u16] = &[SET_WORLD_LEGACY, SET_ORCUST];
// Orcust Harp Horror and World Legacy - "World Crown" to send.
pub const SUPPORT: &[(u32, u16)] = &[(ID, HAND), (57835716, HAND), (57835716, MZONE), (27918365, HAND)];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_DRAW);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_property(duel, EFFECT_FLAG_PLAYER_TARGET);
    e1.set_count_limit_full(duel, 1, ID, EFFECT_COUNT_CODE_OATH);
    e1.set_cost(duel, cost);
    e1.set_target(duel, stock::draw_target::<2>);
    e1.set_operation(duel, stock::draw);
    c.register_effect(duel, e1);
}

fn cfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_WORLD_LEGACY, SET_ORCUST])
        && c.is_monster(duel)
        && (c.is_faceup(duel) || !c.is_location(duel, LOCATION_MZONE))
        && c.is_able_to_grave_as_cost(duel)
}

fn cost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    stock::send_to_grave_cost(duel, a, chk, cfilter, LOCATION_HAND | LOCATION_MZONE)
}
