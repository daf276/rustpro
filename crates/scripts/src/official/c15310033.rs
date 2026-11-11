// Unknown Synchron
use crate::prelude::*;
use crate::testing::{Check, HAND, MZONE, OPPONENT};

const ID: u32 = 15310033;

// Two in hand against the opponent's Elfnote Lucina, with an empty field of its own.
pub const SUPPORT: &[(u32, u16)] = &[(ID, HAND), (ID, HAND), (13597785, MZONE | OPPONENT)];
pub const CHECKS: &[Check] = &[Check::SpecialSummoned];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_code(duel, EFFECT_SPSUMMON_PROC);
    e1.set_property(duel, EFFECT_FLAG_UNCOPYABLE);
    e1.set_range(duel, LOCATION_HAND);
    e1.set_count_limit_full(duel, 1, ID, EFFECT_COUNT_CODE_DUEL);
    e1.set_condition(duel, spcon);
    c.register_effect(duel, e1);
}

fn spcon(duel: &Duel, a: &Args) -> bool {
    let Some(c) = a.c else { return true };
    let tp = c.get_controler(duel);
    duel.get_field_group_count(tp, LOCATION_MZONE, 0) == 0
        && duel.get_field_group_count(tp, 0, LOCATION_MZONE) > 0
        && duel.get_location_count(tp, LOCATION_MZONE) > 0
}
