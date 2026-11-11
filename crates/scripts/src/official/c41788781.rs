// Guldfaxe of the Nordic Beasts
use crate::prelude::*;
use crate::testing::{Check, HAND, MZONE, OPPONENT};

const ID: u32 = 41788781;

// In the hand with the opponent controlling a Synchro Monster (Toxic Boxfish) and nothing on its own field.
pub const SUPPORT: &[(u32, u16)] = &[(ID, HAND), (ID, HAND), (48882106, MZONE | OPPONENT), (48882106, MZONE | OPPONENT)];
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
    c.is_faceup(duel) && c.is_type(duel, TYPE_SYNCHRO)
}

fn spcon(duel: &Duel, a: &Args) -> bool {
    let Some(c) = a.c else { return true };
    let tp = c.get_controler(duel);
    duel.get_location_count(tp, LOCATION_MZONE) > 0
        && duel.get_field_group_count(tp, LOCATION_MZONE, 0) == 0
        && duel.is_existing_matching_card(filter, tp, 0, LOCATION_MZONE, 1, None)
}
