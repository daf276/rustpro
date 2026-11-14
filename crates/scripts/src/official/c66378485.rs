// Neo Flamvell Origin
use crate::prelude::*;
use crate::testing::{Check, HAND, MZONE};

const ID: u32 = 66378485;

pub const LISTED_SERIES: &[u16] = &[SET_FLAMVELL];
// In the hand, with Neo Flamvell Hedgehog and Flamvell Guard on the field.
pub const SUPPORT: &[(u32, u16)] = &[(ID, HAND), (ID, HAND), (2377034, MZONE), (21615956, MZONE)];
pub const CHECKS: &[Check] = &[Check::SpecialSummoned];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_code(duel, EFFECT_SPSUMMON_PROC);
    e1.set_property(duel, EFFECT_FLAG_UNCOPYABLE);
    e1.set_range(duel, LOCATION_HAND);
    e1.set_condition(duel, spcon);
    c.register_effect(duel, e1);
}

fn filter(c: CardId, duel: &Duel) -> bool {
    c.is_faceup(duel) && c.is_set_card(duel, &[SET_FLAMVELL]) && c.get_code(duel) != ID
}

fn spcon(duel: &Duel, a: &Args) -> bool {
    let Some(c) = a.c else { return true };
    let tp = c.get_controler(duel);
    duel.get_location_count(tp, LOCATION_MZONE) > 0
        && duel.is_existing_matching_card(filter, tp, LOCATION_MZONE, 0, 1, None)
        && duel.get_field_group_count(tp, 0, LOCATION_GRAVE) <= 3
}
