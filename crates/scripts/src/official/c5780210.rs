// Creation Resonator
use crate::prelude::*;
use crate::testing::{Check, HAND, MZONE};

const ID: u32 = 5780210;

// In the hand beside a Level 8 Synchro Monster (Red Dragon Archfiend).
pub const SUPPORT: &[(u32, u16)] = &[(ID, HAND), (CARD_RED_DRAGON_ARCHFIEND, MZONE)];
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

fn cfilter(c: CardId, duel: &Duel) -> bool {
    c.is_faceup(duel) && c.is_level_above(duel, 8) && c.is_type(duel, TYPE_SYNCHRO)
}

fn spcon(duel: &Duel, a: &Args) -> bool {
    let Some(c) = a.c else { return true };
    let tp = c.get_controler(duel);
    duel.get_location_count(tp, LOCATION_MZONE) > 0 && duel.is_existing_matching_card(cfilter, tp, LOCATION_MZONE, 0, 1, None)
}
