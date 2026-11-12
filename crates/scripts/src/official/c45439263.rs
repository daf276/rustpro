// Laval Coatl
use crate::prelude::*;
use crate::testing::{Check, GRAVE, HAND};

const ID: u32 = 45439263;

// In the hand, with three other Laval monsters in the Graveyard.
pub const SUPPORT: &[(u32, u16)] = &[(ID, HAND), (2407147, GRAVE), (8041569, GRAVE), (15169262, GRAVE)];
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
    if duel.get_location_count(tp, LOCATION_MZONE) <= 0 {
        return false;
    }
    duel.get_matching_group(|c, duel| c.is_set_card(duel, &[SET_LAVAL]), tp, LOCATION_GRAVE, 0, None)
        .get_class_count(duel, |c, duel| i64::from(c.get_code(duel)))
        >= 3
}
