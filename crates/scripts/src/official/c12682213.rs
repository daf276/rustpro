// Iron Thunder
use crate::prelude::*;
use crate::testing::{OPPONENT, SZONE};

const ID: u32 = 12682213;

// Set, to negate the opponent's set Mystical Space Typhoon.
pub const SUPPORT: &[(u32, u16)] = &[(ID, SZONE), (5318639, SZONE | OPPONENT)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_NEGATE + CATEGORY_DESTROY);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_CHAINING);
    e1.set_condition(duel, condition);
    e1.set_cost(duel, cost);
    e1.set_target(duel, target);
    e1.set_operation(duel, activate);
    c.register_effect(duel, e1);
}

fn condition(duel: &Duel, a: &Args) -> bool {
    let loc: u32 = duel.get_chain_info(a.ev as u8, CHAININFO_TRIGGERING_LOCATION);
    loc & LOCATION_ONFIELD > 0
        && a.re
            .is_some_and(|re| re.is_monster_effect(duel) || re.is_has_type(duel, EFFECT_TYPE_ACTIVATE))
        && duel.is_chain_negatable(a.ev as u8)
}

fn cost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    if chk == 0 {
        return Ok(true);
    }
    let half = duel.get_lp(a.tp) / 2;
    duel.pay_lp_cost(a.tp, half as u32)?;
    Ok(false)
}

fn target(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(true);
    }
    duel.set_operation_info(0, CATEGORY_NEGATE, a.eg, 1, 0, 0);
    let Some(re) = a.re else { return Ok(false) };
    let rc = re.get_handler(duel);
    if rc.is_destructable(duel) && rc.is_relate_to_effect(duel, re) {
        let c = a.e.get_handler(duel);
        let related = a.e.is_has_type(duel, EFFECT_TYPE_ACTIVATE) && c.is_relate_to_effect(duel, a.e);
        // Group.RemoveCard(nil) is a Lua error in the core.
        unported!(duel, !related);
        let mut column = rc.get_column_group(duel);
        column.remove(c);
        duel.set_operation_info(0, CATEGORY_DESTROY, Some(a.eg.unwrap_or_default().union(column)), 1, 0, 0);
    }
    Ok(false)
}

fn activate(duel: &mut Duel, a: &Args) -> Script<()> {
    let Some(re) = a.re else { return Ok(()) };
    let rc = re.get_handler(duel);
    if !(duel.negate_activation(a.ev as u8) && rc.is_relate_to_effect(duel, re)) {
        return Ok(());
    }
    let c = a.e.get_handler(duel);
    let mut column = rc.get_column_group(duel);
    if a.e.is_has_type(duel, EFFECT_TYPE_ACTIVATE) && c.is_relate_to_effect(duel, a.e) {
        column.remove(c);
    }
    if duel.destroy(a.eg.unwrap_or_default(), REASON_EFFECT)? > 0 && !column.is_empty() {
        duel.break_effect()?;
        duel.destroy(column, REASON_EFFECT)?;
    }
    Ok(())
}
