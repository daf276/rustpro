// Trap Eater
use crate::prelude::*;
use crate::testing::{Check, HAND, OPPONENT, SZONE};

const ID: u32 = 13821299;

// In the hand with face-up Traps (Spirit Barrier) on the opponent's side.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (53239672, SZONE | OPPONENT),
    (53239672, SZONE | OPPONENT),
    (53239672, SZONE | OPPONENT),
    (53239672, SZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[Check::SpecialSummoned];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    let e1 = duel.create_effect(c);
    e1.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE + EFFECT_FLAG_UNCOPYABLE);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_code(duel, EFFECT_SPSUMMON_CONDITION);
    e1.set_value(duel, Value::SpSummonCondition(|_, _, _, _, _, _, _| false));
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_FIELD);
    e2.set_code(duel, EFFECT_SPSUMMON_PROC);
    e2.set_property(duel, EFFECT_FLAG_UNCOPYABLE);
    e2.set_range(duel, LOCATION_HAND);
    e2.set_condition(duel, spcon);
    e2.set_target(duel, sptg);
    e2.set_operation(duel, spop);
    c.register_effect(duel, e2);
}

fn spfilter(c: CardId, duel: &Duel) -> bool {
    c.is_faceup(duel) && c.is_trap(duel) && c.is_able_to_grave_as_cost(duel)
}

fn spcon(duel: &Duel, a: &Args) -> bool {
    let Some(c) = a.c else { return true };
    let tp = c.get_controler(duel);
    let g = duel.get_matching_group(spfilter, tp, 0, LOCATION_ONFIELD, None);
    duel.get_location_count(tp, LOCATION_MZONE) > 0
        && !g.is_empty()
        && aux::select_unselect_group_check(duel, g, 1, 1, &aux::chkf_mmz(tp, 1))
}

fn sptg(duel: &mut Duel, a: &Args, _: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let rg = duel.get_matching_group(spfilter, tp, 0, LOCATION_ONFIELD, None);
    let rescon = aux::chkf_mmz(tp, 1);
    let g = aux::select_unselect_group_full(duel, rg, 1, 1, &rescon, tp, HINTMSG_TOGRAVE, Some(&rescon), true)?;
    if g.is_empty() {
        return Ok(false);
    }
    e.set_label_object_group(duel, Some(g));
    Ok(true)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let Some(g) = a.e.get_label_object_group(duel) else { return Ok(()) };
    duel.send_to_grave(g, REASON_COST)?;
    Ok(())
}
