// Skill Drain
use crate::prelude::*;
use crate::testing::{HAND, MZONE, OPPONENT};

const ID: u32 = 82732705;

// In the hand, with an Effect Monster on each side to negate: Sangan.
pub const SUPPORT: &[(u32, u16)] = &[(ID, HAND), (26202165, MZONE), (26202165, MZONE | OPPONENT)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_cost(duel, Cost::pay_lp::<1000>);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_FIELD);
    e2.set_range(duel, LOCATION_SZONE);
    e2.set_target_range(duel, LOCATION_MZONE, LOCATION_MZONE);
    e2.set_target_card(duel, |duel, _, c| {
        c.is_type(duel, TYPE_EFFECT) || c.get_original_type(duel) & TYPE_EFFECT == TYPE_EFFECT
    });
    e2.set_code(duel, EFFECT_DISABLE);
    c.register_effect(duel, e2);
}
