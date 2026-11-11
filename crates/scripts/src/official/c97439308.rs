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
    e1.set_target(duel, stock::draw_target::<2>);
    e1.set_operation(duel, stock::draw);
    c.register_effect(duel, e1);
}

fn condition(duel: &Duel, a: &Args) -> bool {
    duel.get_field_group_count(a.tp, LOCATION_REMOVED, 0) >= 4 && duel.get_field_group_count(a.tp, LOCATION_GRAVE, 0) == 0
}
