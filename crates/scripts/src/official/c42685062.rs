// Emissary from Pandemonium
use crate::prelude::*;
use crate::testing::{Check, HAND, MZONE};

const ID: u32 = 42685062;

// In the hand with a Sangan to Tribute.
pub const SUPPORT: &[(u32, u16)] = &[(ID, HAND), (ID, HAND), (26202165, MZONE), (26202165, MZONE)];
pub const CHECKS: &[Check] = &[Check::TributeSummoned(1)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE + EFFECT_FLAG_UNCOPYABLE);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_code(duel, EFFECT_SPSUMMON_CONDITION);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 0));
    e2.set_type(duel, EFFECT_TYPE_SINGLE);
    e2.set_code(duel, EFFECT_SUMMON_PROC);
    e2.set_property(duel, EFFECT_FLAG_UNCOPYABLE);
    e2.set_condition(duel, sumcon);
    e2.set_operation(duel, sumop);
    e2.set_value(duel, SUMMON_TYPE_TRIBUTE as i32);
    c.register_effect(duel, e2);
}

fn sumcon(duel: &Duel, a: &Args) -> bool {
    let Some(c) = a.c else { return true };
    a.min <= 1 && duel.check_tribute(c, 1, 1, None, c.get_controler(duel), 0x1f)
}

fn sumop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (tp, Some(c)) = (a.tp, a.c) else { return Ok(()) };
    let g = duel.select_tribute(tp, c, 1, 1, None, tp, 0x1f, false)?;
    c.set_material(duel, Some(g));
    duel.release(g, REASON_SUMMON | REASON_MATERIAL)?;
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_code(duel, EFFECT_CHANGE_LEVEL);
    e1.set_value(duel, 5);
    e1.set_reset(duel, RESET_EVENT | (RESETS_STANDARD_DISABLE & !RESET_TOFIELD));
    c.register_effect(duel, e1);
    let e2 = e1.clone_effect(duel);
    e2.set_code(duel, EFFECT_SET_BASE_ATTACK);
    e2.set_value(duel, 1300);
    c.register_effect(duel, e2);
    let e3 = e1.clone_effect(duel);
    e3.set_code(duel, EFFECT_SET_BASE_DEFENSE);
    e3.set_value(duel, 900);
    c.register_effect(duel, e3);
    Ok(())
}
