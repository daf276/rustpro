// Dogoran, the Mad Flame Kaiju
use crate::prelude::*;
use crate::testing::{Check, HAND, MZONE, OPPONENT};

const ID: u32 = 93332803;

pub const COUNTER_LIST: &[u16] = &[COUNTER_KAIJU];

// In the hand with a copy on the opponent's field to Tribute, Kyoutou Waterfront for Kaiju Counters, and Raigeki and Dark Hole to fill it.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, MZONE | OPPONENT),
    (56111151, HAND),
    (12580477, HAND),
    (53129443, HAND),
    (26202165, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(0)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    aux::add_kaiju_procedure(duel, c);
    let e3 = duel.create_effect(c);
    e3.set_category(duel, CATEGORY_DESTROY);
    e3.set_type(duel, EFFECT_TYPE_IGNITION);
    e3.set_range(duel, LOCATION_MZONE);
    e3.set_count_limit(duel, 1, 0);
    e3.set_cost(duel, descost);
    e3.set_target(duel, destg);
    e3.set_operation(duel, desop);
    c.register_effect(duel, e3);
}

fn descost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let (c, tp) = (a.e.get_handler(duel), a.tp);
    if chk == 0 {
        return Ok(duel.is_can_remove_counter(tp, 1, 1, COUNTER_KAIJU, 3, REASON_COST) && c.get_attack_announced_count(duel) == 0);
    }
    duel.remove_counter(tp, 1, 1, COUNTER_KAIJU, 3, REASON_COST)?;
    let e1 = duel.create_effect(c);
    e1.set_description(duel, 3206);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE + EFFECT_FLAG_OATH + EFFECT_FLAG_CLIENT_HINT);
    e1.set_code(duel, EFFECT_CANNOT_ATTACK);
    e1.set_reset(duel, RESETS_STANDARD_PHASE_END);
    c.register_effect(duel, e1);
    Ok(false)
}

fn destg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.get_field_group_count(tp, 0, LOCATION_MZONE) > 0);
    }
    let g = duel.get_field_group(tp, 0, LOCATION_MZONE);
    duel.set_operation_info(0, CATEGORY_DESTROY, Some(g), g.len() as u8, 0, 0);
    Ok(false)
}

fn desop(duel: &mut Duel, a: &Args) -> Script<()> {
    let g = duel.get_field_group(a.tp, 0, LOCATION_MZONE);
    duel.destroy(g, REASON_EFFECT)?;
    Ok(())
}
