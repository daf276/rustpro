// Dinomorphia Intact
use crate::prelude::*;
use crate::testing::{Check, GRAVE, HAND, SZONE};

const ID: u32 = 7336745;

pub const LISTED_SERIES: &[u16] = &[SET_DINOMORPHIA];
// Set, with Dinomorphia Shell to summon a Dinomorphia Token on the opponent's Battle Phase, and a copy in the GY.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, SZONE),
    (ID, SZONE),
    (ID, GRAVE),
    (25419323, SZONE),
    (25419323, SZONE),
    (25419323, HAND),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_NEGATE + CATEGORY_DESTROY);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_CHAINING);
    e1.set_count_limit_full(duel, 1, ID, EFFECT_COUNT_CODE_OATH);
    e1.set_condition(duel, condition);
    e1.set_cost(duel, cost);
    e1.set_target(duel, stock::negate_destroy_target);
    e1.set_operation(duel, activate);
    c.register_effect(duel, e1);
    register_no_battle_damage(duel, c, ID);
}

// The Graveyard effect of Intact, Sonic and Shell: banish this card so its controller takes no battle damage.
pub fn register_no_battle_damage(duel: &mut Duel, c: CardId, id: u32) {
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(id, 1));
    e2.set_type(duel, EFFECT_TYPE_QUICK_O);
    e2.set_code(duel, EVENT_PRE_DAMAGE_CALCULATE);
    e2.set_range(duel, LOCATION_GRAVE);
    e2.set_condition(duel, nodamcon);
    e2.set_cost(duel, Cost::self_banish);
    e2.set_operation(duel, nodamop);
    c.register_effect(duel, e2);
}

fn condition(duel: &Duel, a: &Args) -> bool {
    a.re.is_some_and(|re| re.is_monster_effect(duel))
        && duel.is_chain_negatable(a.ev as u8)
        && duel.is_existing_matching_card(
            aux::faceup_filter(|c, duel| c.is_set_card(duel, &[SET_DINOMORPHIA])),
            a.tp,
            LOCATION_ONFIELD,
            0,
            1,
            None,
        )
}

pub fn cost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    if chk == 0 {
        return Ok(true);
    }
    let lp = duel.get_lp(a.tp);
    duel.pay_lp_cost(a.tp, (lp / 2) as u32)?;
    Ok(false)
}

fn activate(duel: &mut Duel, a: &Args) -> Script<()> {
    let e1 = duel.create_effect(a.e.get_handler(duel));
    e1.set_description(duel, aux::stringid(ID, 2));
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_property(duel, EFFECT_FLAG_PLAYER_TARGET + EFFECT_FLAG_CLIENT_HINT);
    e1.set_code(duel, EFFECT_CHANGE_BATTLE_DAMAGE);
    e1.set_target_range(duel, 1, 0);
    // The core rounds the Lua's LP/2.
    e1.set_value(duel, Value::Own(|duel, e| (duel.get_lp(e.get_handler_player(duel)) + 1) / 2));
    e1.set_reset(duel, RESET_PHASE | PHASE_END);
    duel.register_effect(e1, a.tp);
    let Some(re) = a.re else { return Ok(()) };
    if duel.negate_activation(a.ev as u8) && re.get_handler(duel).is_relate_to_effect(duel, re) {
        duel.destroy(a.eg.unwrap_or_default(), REASON_EFFECT)?;
    }
    Ok(())
}

fn nodamcon(duel: &Duel, a: &Args) -> bool {
    let tp = a.tp;
    duel.get_lp(tp) <= 2000 && duel.get_battle_damage(tp) > 0 && duel.is_player_affected_by_effect(tp, EFFECT_AVOID_BATTLE_DAMAGE).is_none()
}

fn nodamop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    if duel.is_player_affected_by_effect(tp, EFFECT_AVOID_BATTLE_DAMAGE).is_some() {
        return Ok(());
    }
    let e1 = duel.create_effect(a.e.get_handler(duel));
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_code(duel, EFFECT_AVOID_BATTLE_DAMAGE);
    e1.set_property(duel, EFFECT_FLAG_PLAYER_TARGET);
    e1.set_target_range(duel, 1, 0);
    e1.set_reset(duel, RESET_PHASE | PHASE_DAMAGE);
    duel.register_effect(e1, tp);
    Ok(())
}
