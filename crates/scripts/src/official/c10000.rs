// Ten Thousand Dragon
use crate::prelude::*;
use crate::testing::{Check, HAND, MZONE};

const ID: u32 = 10000;

// A copy in the hand, Special Summoned by Tributing monsters with 10000 ATK and DEF in total.
pub const SUPPORT: &[(u32, u16)] = &[(ID, HAND), (96345184, MZONE), (50251045, MZONE)];
pub const CHECKS: &[Check] = &[Check::SpecialSummoned];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE + EFFECT_FLAG_UNCOPYABLE);
    e1.set_code(duel, EFFECT_SPSUMMON_CONDITION);
    e1.set_value(duel, Value::SpSummonCondition(|_, _, _, _, _, _, _| false));
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 0));
    e2.set_type(duel, EFFECT_TYPE_FIELD);
    e2.set_code(duel, EFFECT_SPSUMMON_PROC);
    e2.set_property(duel, EFFECT_FLAG_UNCOPYABLE);
    e2.set_range(duel, LOCATION_HAND);
    e2.set_value(duel, 1);
    e2.set_condition(duel, spcon);
    e2.set_operation(duel, spop);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_type(duel, EFFECT_TYPE_SINGLE);
    e3.set_property(duel, EFFECT_FLAG_SINGLE_RANGE);
    e3.set_range(duel, LOCATION_MZONE);
    e3.set_code(duel, EFFECT_SET_ATTACK);
    e3.set_condition(duel, atkcon);
    e3.set_value(duel, 10000);
    c.register_effect(duel, e3);
    let e4 = e3.clone_effect(duel);
    e4.set_code(duel, EFFECT_SET_DEFENSE);
    c.register_effect(duel, e4);
}

fn rescon(duel: &Duel, sg: CardSet, tp: u8) -> (bool, bool) {
    let total = sg.get_sum(duel, |c, duel| c.get_attack(duel).into()) + sg.get_sum(duel, |c, duel| c.get_defense(duel).into());
    (duel.get_mzone_count(tp, Some(sg)) >= 1 && total >= 10000, false)
}

fn spcon(duel: &Duel, a: &Args) -> bool {
    let Some(c) = a.c else { return true };
    let tp = c.get_controler(duel);
    aux::select_unselect_group_check(duel, duel.get_release_group(tp), 1, 7, &|duel, sg, _, _| rescon(duel, sg, tp))
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    let rg = duel.get_release_group(tp);
    let rescon = |duel: &Duel, sg, _, _| rescon(duel, sg, tp);
    let g = aux::select_unselect_group_full(duel, rg, 1, 7, &rescon, tp, HINTMSG_RELEASE, Some(&rescon), false)?;
    duel.release(g, REASON_COST)?;
    Ok(())
}

fn atkcon(duel: &Duel, a: &Args) -> bool {
    a.e.get_handler(duel).get_summon_type(duel) == SUMMON_TYPE_SPECIAL + 1
}
