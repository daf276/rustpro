// Legendary Six Samurai - Kizan
use crate::prelude::*;
use crate::testing::{Check, HAND, MZONE};

const ID: u32 = 49721904;

pub const LISTED_SERIES: &[u16] = &[SET_SIX_SAMURAI];
// Six Samurai (Kageki, Hatsume) on the field to Special Summon from the hand and to boost it.
pub const SUPPORT: &[(u32, u16)] = &[(ID, HAND), (ID, HAND), (2511717, MZONE), (44686185, MZONE)];
pub const CHECKS: &[Check] = &[Check::SpecialSummoned];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_code(duel, EFFECT_SPSUMMON_PROC);
    e1.set_property(duel, EFFECT_FLAG_UNCOPYABLE);
    e1.set_range(duel, LOCATION_HAND);
    e1.set_condition(duel, spcon);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_SINGLE);
    e2.set_code(duel, EFFECT_UPDATE_ATTACK);
    e2.set_property(duel, EFFECT_FLAG_SINGLE_RANGE);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_condition(duel, valcon);
    e2.set_value(duel, 300);
    c.register_effect(duel, e2);
    let e3 = e2.clone_effect(duel);
    e3.set_code(duel, EFFECT_UPDATE_DEFENSE);
    c.register_effect(duel, e3);
}

fn spfilter(c: CardId, duel: &Duel) -> bool {
    c.is_faceup(duel) && c.is_set_card(duel, &[SET_SIX_SAMURAI]) && c.get_code(duel) != ID
}

fn spcon(duel: &Duel, a: &Args) -> bool {
    let Some(c) = a.c else { return true };
    let tp = c.get_controler(duel);
    duel.get_location_count(tp, LOCATION_MZONE) > 0 && duel.is_existing_matching_card(spfilter, tp, LOCATION_MZONE, 0, 1, None)
}

fn vfilter(c: CardId, duel: &Duel) -> bool {
    c.is_faceup(duel) && c.is_set_card(duel, &[SET_SIX_SAMURAI])
}

fn valcon(duel: &Duel, a: &Args) -> bool {
    let c = a.e.get_handler(duel);
    duel.is_existing_matching_card(vfilter, c.get_controler(duel), LOCATION_MZONE, 0, 2, Some(c.into()))
}
