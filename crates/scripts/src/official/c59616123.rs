// Trap Stun
use engine::constants::MSG_CHAIN_DISABLED;

use crate::prelude::*;
use crate::testing::{Check, OPPONENT, SZONE};

const ID: u32 = 59616123;

// Set, with Traps in the Spell & Trap Zones of both players to negate.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, SZONE),
    (ID, SZONE),
    (4206964, SZONE),
    (14315573, SZONE),
    (4206964, SZONE | OPPONENT),
    (14315573, SZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[Check::Resolves, Check::Message(MSG_CHAIN_DISABLED)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_operation(duel, activate);
    c.register_effect(duel, e1);
}

fn activate(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_code(duel, EFFECT_DISABLE);
    e1.set_target_range(duel, LOCATION_SZONE, LOCATION_SZONE);
    e1.set_target_card(duel, distarget);
    e1.set_reset(duel, RESET_PHASE | PHASE_END);
    duel.register_effect(e1, tp);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_CONTINUOUS);
    e2.set_code(duel, EVENT_CHAIN_SOLVING);
    e2.set_operation(duel, disoperation);
    e2.set_reset(duel, RESET_PHASE | PHASE_END);
    duel.register_effect(e2, tp);
    let e3 = duel.create_effect(c);
    e3.set_type(duel, EFFECT_TYPE_FIELD);
    e3.set_code(duel, EFFECT_DISABLE_TRAPMONSTER);
    e3.set_target_range(duel, LOCATION_MZONE, LOCATION_MZONE);
    e3.set_target_card(duel, distarget);
    e3.set_reset(duel, RESET_PHASE | PHASE_END);
    duel.register_effect(e3, tp);
    Ok(())
}

fn distarget(duel: &Duel, e: EffectId, c: CardId) -> bool {
    c != e.get_handler(duel) && c.is_trap(duel)
}

fn disoperation(duel: &mut Duel, a: &Args) -> Script<()> {
    let ev = a.ev as u8;
    let tl: u32 = duel.get_chain_info(ev, CHAININFO_TRIGGERING_LOCATION);
    if tl == LOCATION_SZONE && a.re.is_some_and(|re| re.is_trap_effect(duel)) {
        duel.negate_effect(ev);
    }
    Ok(())
}
