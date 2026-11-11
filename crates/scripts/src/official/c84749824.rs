// Solemn Warning
use crate::prelude::*;

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_category(duel, CATEGORY_DISABLE_SUMMON + CATEGORY_DESTROY);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_SUMMON);
    e1.set_condition(duel, |duel, _| duel.get_current_chain_full(true) == 0);
    e1.set_cost(duel, cost);
    e1.set_target(duel, target1);
    e1.set_operation(duel, activate1);
    c.register_effect(duel, e1);
    let e2 = e1.clone_effect(duel);
    e2.set_code(duel, EVENT_FLIP_SUMMON);
    c.register_effect(duel, e2);
    let e3 = e1.clone_effect(duel);
    e3.set_code(duel, EVENT_SPSUMMON);
    c.register_effect(duel, e3);
    let e4 = duel.create_effect(c);
    e4.set_category(duel, CATEGORY_NEGATE + CATEGORY_DESTROY);
    e4.set_property(duel, EFFECT_FLAG_DAMAGE_STEP + EFFECT_FLAG_DAMAGE_CAL);
    e4.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e4.set_code(duel, EVENT_CHAINING);
    e4.set_condition(duel, condition2);
    e4.set_cost(duel, cost);
    e4.set_target(duel, stock::negate_destroy_target);
    e4.set_operation(duel, activate2);
    c.register_effect(duel, e4);
}

fn condition2(duel: &Duel, a: &Args) -> bool {
    if !duel.is_chain_negatable(a.ev as u8) {
        return false;
    }
    a.re.is_some_and(|re| {
        (re.is_monster_effect(duel) || re.is_has_type(duel, EFFECT_TYPE_ACTIVATE)) && re.is_has_category(duel, CATEGORY_SPECIAL_SUMMON)
    })
}

// cost1 and cost2, which are the same.
fn cost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    if chk == 0 {
        return Ok(duel.check_lp_cost(a.tp, 2000));
    }
    duel.pay_lp_cost(a.tp, 2000)?;
    Ok(false)
}

fn target1(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(true);
    }
    let count = a.eg.map_or(0, |eg| eg.len()) as u8;
    duel.set_operation_info(0, CATEGORY_DISABLE_SUMMON, a.eg, count, 0, 0);
    duel.set_operation_info(0, CATEGORY_DESTROY, a.eg, count, 0, 0);
    Ok(false)
}

fn activate1(duel: &mut Duel, a: &Args) -> Script<()> {
    let eg = a.eg.unwrap_or_default();
    duel.negate_summon(eg);
    duel.destroy(eg, REASON_EFFECT)?;
    Ok(())
}

fn activate2(duel: &mut Duel, a: &Args) -> Script<()> {
    if duel.negate_activation(a.ev as u8)
        && let Some(re) = a.re
        && re.get_handler(duel).is_relate_to_effect(duel, re)
    {
        duel.destroy(a.eg.unwrap_or_default(), REASON_EFFECT)?;
    }
    Ok(())
}
