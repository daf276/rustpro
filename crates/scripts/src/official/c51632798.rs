// Fusilier Dragon, the Dual-Mode Beast
use crate::prelude::*;
use crate::testing::{Check, HAND};

const ID: u32 = 51632798;

pub const SUPPORT: &[(u32, u16)] = &[(ID, HAND), (ID, HAND), (ID, HAND)];
pub const CHECKS: &[Check] = &[Check::TributeSummoned(0), Check::TributeSet(0)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE + EFFECT_FLAG_UNCOPYABLE);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_code(duel, EFFECT_SUMMON_PROC);
    e1.set_condition(duel, ntcon);
    e1.set_operation(duel, ntop);
    c.register_effect(duel, e1);
    let e2 = e1.clone_effect(duel);
    e2.set_code(duel, EFFECT_SET_PROC);
    c.register_effect(duel, e2);
}

fn ntcon(duel: &Duel, a: &Args) -> bool {
    let Some(c) = a.c else { return true };
    a.min == 0 && c.get_level(duel) > 4 && duel.get_location_count(c.get_controler(duel), LOCATION_MZONE) > 0
}

fn ntop(duel: &mut Duel, a: &Args) -> Script<()> {
    let Some(c) = a.c else { return Ok(()) };
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_property(duel, EFFECT_FLAG_SINGLE_RANGE);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_reset(duel, RESET_EVENT | (RESETS_STANDARD_DISABLE & !RESET_TOFIELD));
    e1.set_code(duel, EFFECT_SET_BASE_ATTACK);
    e1.set_value(duel, 1400);
    c.register_effect(duel, e1);
    let e2 = e1.clone_effect(duel);
    e2.set_code(duel, EFFECT_SET_BASE_DEFENSE);
    e2.set_value(duel, 1000);
    c.register_effect(duel, e2);
    Ok(())
}
