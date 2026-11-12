// Cockadoodledoo
use crate::prelude::*;
use crate::testing::{Check, HAND, MZONE, OPPONENT};

const ID: u32 = 42338879;

// In the hand, with an opponent's monster for the Level 4 procedure and Dark Hole to clear the field for the Level 3 one.
pub const SUPPORT: &[(u32, u16)] = &[(ID, HAND), (ID, HAND), (26202165, MZONE | OPPONENT), (53129443, HAND | OPPONENT)];
// Both procedures form no chain link.
pub const CHECKS: &[Check] = &[Check::SpecialSummoned];

// The Lua's label object, a function that assumes the Level for "cannot Special Summon" targets, is left out: the engine passes them no procedure effect.
pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_property(duel, EFFECT_FLAG_UNCOPYABLE);
    e1.set_code(duel, EFFECT_SPSUMMON_PROC);
    e1.set_range(duel, LOCATION_HAND);
    e1.set_condition(duel, lv3spcon);
    e1.set_operation(duel, spop::<3>);
    c.register_effect(duel, e1);
    let e2 = e1.clone_effect(duel);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_condition(duel, lv4spcon);
    e2.set_operation(duel, spop::<4>);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_type(duel, EFFECT_TYPE_SINGLE);
    e3.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE);
    e3.set_code(duel, EFFECT_LEAVE_FIELD_REDIRECT);
    e3.set_condition(duel, |duel, a| a.e.get_handler(duel).is_faceup(duel));
    e3.set_value(duel, LOCATION_REMOVED as i32);
    c.register_effect(duel, e3);
}

fn lv3spcon(duel: &Duel, a: &Args) -> bool {
    let Some(c) = a.c else { return true };
    duel.get_location_count(c.get_controler(duel), LOCATION_MZONE) > 0 && duel.get_field_group_count(0, LOCATION_MZONE, LOCATION_MZONE) == 0
}

fn lv4spcon(duel: &Duel, a: &Args) -> bool {
    let Some(c) = a.c else { return true };
    let tp = c.get_controler(duel);
    duel.get_location_count(tp, LOCATION_MZONE) > 0
        && duel.get_field_group_count(tp, 0, LOCATION_MZONE) > 0
        && duel.get_field_group_count(tp, LOCATION_ONFIELD, 0) == 0
}

fn spop<const LV: u32>(duel: &mut Duel, a: &Args) -> Script<()> {
    let Some(c) = a.c else { return Ok(()) };
    c.assume_property(duel, ASSUME_LEVEL, LV.into());
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_code(duel, EFFECT_CHANGE_LEVEL);
    e1.set_value(duel, LV as i32);
    e1.set_reset(duel, RESET_EVENT | (RESETS_STANDARD_DISABLE & !RESET_TOFIELD));
    c.register_effect(duel, e1);
    Ok(())
}
