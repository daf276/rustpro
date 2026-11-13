// Gatling Ogre
use crate::prelude::*;
use crate::testing::{Check, MZONE, OPPONENT, SZONE};

const ID: u32 = 88931417;

// Set Traps (Trap Hole) to send as the cost, and the opponent's attackers (Elfnote Lucina) to target it and to count for its second effect.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (ID, MZONE),
    (4206964, SZONE),
    (4206964, SZONE),
    (4206964, SZONE),
    (4206964, SZONE),
    (13597785, MZONE | OPPONENT),
    (13597785, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_DAMAGE);
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_property(duel, EFFECT_FLAG_PLAYER_TARGET);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_count_limit(duel, 3, ID);
    e1.set_cost(duel, damcost);
    e1.set_target(duel, damtg);
    e1.set_operation(duel, damop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_DAMAGE);
    e2.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e2.set_code(duel, EVENT_BE_BATTLE_TARGET);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_count_limit(duel, 1, 0);
    e2.set_target(duel, damendbptg);
    e2.set_operation(duel, damendbpop);
    c.register_effect(duel, e2);
}

fn costfilter(c: CardId, duel: &Duel) -> bool {
    c.is_facedown(duel) && c.is_able_to_grave_as_cost(duel)
}

fn damcost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let tp = a.tp;
    let g = duel.get_matching_group(costfilter, tp, LOCATION_STZONE, 0, None);
    if chk == 0 {
        return Ok(!g.is_empty());
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TOGRAVE);
    let g = duel.select_matching_card(tp, costfilter, tp, LOCATION_STZONE, 0, 1, g.len() as u8, None)?;
    a.e.set_label(duel, g.len() as i64);
    duel.send_to_grave(g, REASON_COST)?;
    Ok(false)
}

fn damtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(true);
    }
    let tp = a.tp;
    let dam = a.e.get_label(duel) as i32 * 800;
    duel.set_target_player(1 - tp);
    duel.set_target_param(dam as u32);
    duel.set_operation_info(0, CATEGORY_DAMAGE, None, 0, 1 - tp, dam);
    Ok(false)
}

fn damop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    let p: u8 = duel.get_chain_info(0, CHAININFO_TARGET_PLAYER);
    let d: u32 = duel.get_chain_info(0, CHAININFO_TARGET_PARAM);
    duel.damage(p, d, REASON_EFFECT)?;
    let e1 = duel.create_effect(a.e.get_handler(duel));
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_property(duel, EFFECT_FLAG_PLAYER_TARGET);
    e1.set_code(duel, EFFECT_CHANGE_DAMAGE);
    e1.set_target_range(duel, 0, 1);
    e1.set_value(duel, Value::ChangeDamage(damval));
    e1.set_reset(duel, RESET_PHASE | PHASE_END);
    duel.register_effect(e1, tp);
    Ok(())
}

fn damval(duel: &mut Duel, _: EffectId, re: Option<EffectId>, val: u32, r: u32, _: u8, _: Option<CardId>) -> u32 {
    match re {
        Some(re) if r & REASON_EFFECT > 0 && re.is_monster_effect(duel) && re.get_handler(duel).is_race_except(duel, RACE_FIEND) => 0,
        _ => val,
    }
}

fn damendbptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    let ct = duel.get_matching_group_count(CardId::is_attack_pos, tp, 0, LOCATION_MZONE, None);
    if chk == 0 {
        return Ok(ct > 0);
    }
    duel.set_operation_info(0, CATEGORY_DAMAGE, None, 0, 1 - tp, ct as i32 * 500);
    Ok(false)
}

fn damendbpop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    let dam = 500 * duel.get_matching_group_count(CardId::is_attack_pos, tp, 0, LOCATION_MZONE, None) as u32;
    if dam > 0 && duel.damage(1 - tp, dam, REASON_EFFECT)? > 0 {
        duel.break_effect()?;
        duel.skip_phase(1 - tp, PHASE_BATTLE, RESET_PHASE | PHASE_BATTLE_STEP, 1);
    }
    Ok(())
}
