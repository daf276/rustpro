// Dinowrestler Coelasilat
use crate::prelude::*;
use crate::testing::{Check, HAND};

const ID: u32 = 75366958;

pub const SUPPORT: &[(u32, u16)] = &[(ID, HAND), (ID, HAND)];
pub const CHECKS: &[Check] = &[Check::SpecialSummoned];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_code(duel, EFFECT_SPSUMMON_PROC);
    e1.set_property(duel, EFFECT_FLAG_UNCOPYABLE);
    e1.set_range(duel, LOCATION_HAND);
    e1.set_count_limit_full(duel, 1, ID, EFFECT_COUNT_CODE_DUEL);
    e1.set_condition(duel, spcon);
    e1.set_operation(duel, spop);
    c.register_effect(duel, e1);
}

fn spcon(duel: &Duel, a: &Args) -> bool {
    let Some(c) = a.c else { return true };
    let tp = c.get_controler(duel);
    duel.get_field_group_count(tp, LOCATION_MZONE, 0) == 0 && duel.get_location_count(tp, LOCATION_MZONE) > 0
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let Some(c) = a.c else { return Ok(()) };
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_code(duel, EFFECT_CANNOT_BE_LINK_MATERIAL);
    e1.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE + EFFECT_FLAG_UNCOPYABLE);
    e1.set_value(duel, Value::Card(synlimit));
    e1.set_reset(duel, RESET_EVENT | (RESETS_STANDARD_DISABLE & !RESET_TOFIELD));
    c.register_effect(duel, e1);
    Ok(())
}

fn synlimit(duel: &Duel, _: EffectId, c: CardId) -> i32 {
    i32::from(!c.is_set_card(duel, &[SET_DINOWRESTLER]))
}
