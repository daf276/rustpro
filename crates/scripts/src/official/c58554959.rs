// Evil HERO Malicious Edge
use crate::prelude::*;
use crate::testing::{Check, HAND, MZONE, OPPONENT};

const ID: u32 = 58554959;

// In the hand with a Sangan to Tribute and one of the opponent's, for the 1-Tribute Summon.
pub const SUPPORT: &[(u32, u16)] = &[(ID, HAND), (26202165, MZONE), (26202165, MZONE | OPPONENT)];
pub const CHECKS: &[Check] = &[Check::TributeSummoned(1)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE + EFFECT_FLAG_UNCOPYABLE);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_code(duel, EFFECT_SUMMON_PROC);
    e1.set_condition(duel, otcon);
    e1.set_operation(duel, otop);
    e1.set_value(duel, SUMMON_TYPE_TRIBUTE as i32);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_SINGLE);
    e2.set_code(duel, EFFECT_PIERCE);
    c.register_effect(duel, e2);
}

fn otcon(duel: &Duel, a: &Args) -> bool {
    let Some(c) = a.c else { return true };
    let tp = c.get_controler(duel);
    duel.get_field_group_count(tp, 0, LOCATION_MZONE) > 0
        && c.get_level(duel) > 6
        && a.min <= 1
        && duel.check_tribute(c, 1, 1, None, tp, 0x1f)
}

fn otop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (tp, Some(c)) = (a.tp, a.c) else { return Ok(()) };
    let sg = duel.select_tribute(tp, c, 1, 1, None, tp, 0x1f, false)?;
    c.set_material(duel, Some(sg));
    duel.release(sg, REASON_SUMMON | REASON_MATERIAL)?;
    Ok(())
}
