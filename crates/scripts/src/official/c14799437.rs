// Infernoid Onuncu
use crate::prelude::*;
use crate::testing::{Check, GRAVE, HAND, OPPONENT};

const ID: u32 = 14799437;

pub const LISTED_SERIES: &[u16] = &[SET_INFERNOID];

// Infernoid Decatron and Pirmais in the Graveyard to banish for its Special Summon, and the opponent's Raigeki and Dark Hole to
// negate.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, GRAVE),
    (58446973, GRAVE),
    (58446973, GRAVE),
    (60551528, GRAVE),
    (60551528, GRAVE),
    (12580477, HAND | OPPONENT),
    (53129443, HAND | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Infernoid::register_summon_procedure::<3>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_DESTROY);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e1.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e1.set_target(duel, destg);
    e1.set_operation(duel, desop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_NEGATE + CATEGORY_REMOVE);
    e2.set_type(duel, EFFECT_TYPE_QUICK_O);
    e2.set_property(duel, EFFECT_FLAG_DAMAGE_STEP + EFFECT_FLAG_DAMAGE_CAL);
    e2.set_code(duel, EVENT_CHAINING);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_count_limit(duel, 1, 0);
    e2.set_condition(duel, negcon);
    e2.set_cost(duel, negcost);
    e2.set_target(duel, super::c30350202::negtg);
    e2.set_operation(duel, super::c30350202::negop);
    c.register_effect(duel, e2);
    aux::double_snare_validity(duel, c, LOCATION_MZONE);
}

fn destg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    let c = a.e.get_handler(duel);
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(|_, _| true, tp, LOCATION_MZONE, LOCATION_MZONE, 1, Some(c.into())));
    }
    let g = duel.get_matching_group(|_, _| true, tp, LOCATION_MZONE, LOCATION_MZONE, Some(c.into()));
    duel.set_operation_info(0, CATEGORY_DESTROY, Some(g), g.len() as u8, 0, 0);
    Ok(false)
}

fn desop(duel: &mut Duel, a: &Args) -> Script<()> {
    let c = a.e.get_handler(duel);
    let g = duel.get_matching_group(|_, _| true, a.tp, LOCATION_MZONE, LOCATION_MZONE, Some(c.into()));
    if !g.is_empty() {
        duel.destroy(g, REASON_EFFECT)?;
    }
    Ok(())
}

fn negcon(duel: &Duel, a: &Args) -> bool {
    let Some(re) = a.re else { return false };
    !a.e.get_handler(duel).is_status(duel, STATUS_BATTLE_DESTROYED) && re.is_spell_trap_effect(duel) && duel.is_chain_negatable(a.ev as u8)
}

fn cfilter(c: CardId, duel: &Duel) -> bool {
    !c.is_status(duel, STATUS_BATTLE_DESTROYED)
}

pub fn negcost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.check_release_group_cost(tp, cfilter, (1, 1), false, None, None));
    }
    let g = duel.select_release_group_cost(tp, cfilter, (1, 1), false, None, None)?;
    duel.release(g, REASON_COST)?;
    Ok(false)
}
