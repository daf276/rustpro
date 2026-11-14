// Heraldic Beast Eale
use crate::prelude::*;
use crate::testing::{Check, HAND, MZONE};

const ID: u32 = 82315772;

pub const LISTED_SERIES: &[u16] = &[SET_HERALDIC_BEAST];
// In the hand beside two other Heraldic Beast monsters (Eale itself) on the field.
pub const SUPPORT: &[(u32, u16)] = &[(ID, HAND), (ID, MZONE), (ID, MZONE)];
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

fn spcon(duel: &Duel, a: &Args) -> bool {
    let Some(c) = a.c else { return true };
    let tp = c.get_controler(duel);
    duel.get_location_count(tp, LOCATION_MZONE) > 0
        && duel.is_existing_matching_card(
            aux::faceup_filter(|c, duel| c.is_set_card(duel, &[SET_HERALDIC_BEAST])),
            tp,
            LOCATION_MZONE,
            0,
            2,
            None,
        )
}
