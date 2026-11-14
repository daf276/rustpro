// Battlewasp - Halberd the Charge
use super::c71456737::Procedure;
use crate::prelude::*;
use crate::testing::{Check, MZONE, OPPONENT};

const ID: u32 = 53950487;

pub const LISTED_SERIES: &[u16] = &[SET_BATTLEWASP];
// Copies on the field to attack the opponent's Dark Magician, Summoned Skull, Black Luster Soldier and Steam Knight - Hamburk
// (ATK 2500 or more) or Sangan (weaker, for the battle damage).
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (ID, MZONE),
    (26202165, MZONE | OPPONENT),
    (36996508, MZONE | OPPONENT),
    (70781052, MZONE | OPPONENT),
    (10000100, MZONE | OPPONENT),
    (23923758, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    Synchro::add_procedure::<Procedure>(duel, c);
    c.enable_revive_limit(duel);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_ATKCHANGE);
    e1.set_type(duel, EFFECT_TYPE_QUICK_O);
    e1.set_code(duel, EVENT_PRE_DAMAGE_CALCULATE);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_condition(duel, atkcon);
    e1.set_cost(duel, atkcost);
    e1.set_operation(duel, atkop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_DAMAGE);
    e2.set_property(duel, EFFECT_FLAG_PLAYER_TARGET);
    e2.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e2.set_code(duel, EVENT_BATTLE_DAMAGE);
    e2.set_count_limit(duel, 1, ID);
    e2.set_condition(duel, |_, a| a.ep != a.tp);
    e2.set_target(duel, damtg);
    e2.set_operation(duel, damop);
    c.register_effect(duel, e2);
}

fn atkcon(duel: &Duel, a: &Args) -> bool {
    let c = a.e.get_handler(duel);
    let d = c.get_battle_target(duel);
    Some(c) == duel.get_attacker()
        && d.is_some_and(|d| d.is_faceup(duel) && !d.is_controler(duel, a.tp) && d.get_attack(duel) >= c.get_attack(duel))
}

fn atkcost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let c = a.e.get_handler(duel);
    if chk == 0 {
        return Ok(c.get_flag_effect(duel, ID) == 0);
    }
    c.register_flag_effect(duel, ID, RESET_CHAIN, 0, 1);
    Ok(false)
}

fn atkop(duel: &mut Duel, a: &Args) -> Script<()> {
    let d = duel.get_attacker().and_then(|c| c.get_battle_target(duel));
    unported!(duel, d.is_none());
    let Some(d) = d else { return Ok(()) };
    if d.is_relate_to_battle(duel) && d.is_faceup(duel) {
        let e1 = duel.create_effect(a.e.get_handler(duel));
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_code(duel, EFFECT_SET_ATTACK_FINAL);
        e1.set_reset(duel, RESET_PHASE | PHASE_DAMAGE_CAL);
        e1.set_value(duel, (d.get_attack(duel) + 1) / 2);
        d.register_effect(duel, e1);
    }
    Ok(())
}

fn damfilter(c: CardId, duel: &Duel) -> bool {
    c.is_faceup(duel) && c.is_set_card(duel, &[SET_BATTLEWASP])
}

fn damtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(damfilter, tp, LOCATION_MZONE, 0, 1, None));
    }
    let val = duel.get_matching_group_count(damfilter, tp, LOCATION_MZONE, 0, None) as u32 * 200;
    duel.set_target_player(1 - tp);
    duel.set_target_param(val);
    duel.set_operation_info(0, CATEGORY_DAMAGE, None, 0, 1 - tp, val as i32);
    Ok(false)
}

fn damop(duel: &mut Duel, a: &Args) -> Script<()> {
    let p: u8 = duel.get_chain_info(0, CHAININFO_TARGET_PLAYER);
    let val = duel.get_matching_group_count(damfilter, a.tp, LOCATION_MZONE, 0, None) as u32 * 200;
    duel.damage(p, val, REASON_EFFECT)?;
    Ok(())
}
