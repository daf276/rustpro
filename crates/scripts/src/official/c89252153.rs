// Elemental HERO Necroshade
use crate::prelude::*;
use crate::testing::{Check, GRAVE, HAND};

const ID: u32 = 89252153;

// A copy in the hand (Level 5, so it qualifies) to Normal Summon without Tributes by a copy in the Graveyard.
pub const SUPPORT: &[(u32, u16)] = &[(ID, HAND), (ID, GRAVE)];
pub const CHECKS: &[Check] = &[Check::TributeSummoned(0)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_range(duel, LOCATION_GRAVE);
    e1.set_target_range(duel, LOCATION_HAND, 0);
    e1.set_code(duel, EFFECT_SUMMON_PROC);
    e1.set_property(duel, EFFECT_FLAG_NO_TURN_RESET);
    e1.set_count_limit(duel, 1, 0);
    e1.set_condition(duel, ntcon);
    e1.set_target_card(duel, nttg);
    c.register_effect(duel, e1);
}

fn ntcon(duel: &Duel, a: &Args) -> bool {
    let Some(c) = a.c else { return true };
    a.min == 0 && duel.get_location_count(c.get_controler(duel), LOCATION_MZONE) > 0
}

fn nttg(duel: &Duel, _: EffectId, c: CardId) -> bool {
    c.is_level_above(duel, 5) && c.is_set_card(duel, &[SET_ELEMENTAL_HERO])
}
