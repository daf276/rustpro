// Jinzo
use engine::constants::MSG_ATTACK;

use crate::prelude::*;
use crate::testing::{Check, HAND, MZONE, OPPONENT, SZONE};

const ID: u32 = 77585513;

// On the field, attacking, with Traps (Trap Hole, Mirror Force, Infinite Impermanence) Set on both sides and in the hands.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (4206964, SZONE),
    (44095762, SZONE | OPPONENT),
    (10045474, SZONE | OPPONENT),
    (10045474, HAND | OPPONENT),
    (4206964, HAND),
];
pub const CHECKS: &[Check] = &[Check::Message(MSG_ATTACK)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_code(duel, EFFECT_CANNOT_TRIGGER);
    e1.set_property(duel, EFFECT_FLAG_SET_AVAILABLE);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_target_range(duel, LOCATION_HAND | LOCATION_SZONE, LOCATION_HAND | LOCATION_SZONE);
    e1.set_target_card(duel, |duel, _, c| c.is_trap(duel));
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_FIELD);
    e2.set_code(duel, EFFECT_CANNOT_ACTIVATE);
    e2.set_property(duel, EFFECT_FLAG_PLAYER_TARGET);
    e2.set_target_range(duel, 1, 1);
    e2.set_value(duel, Value::EffectPlayer(aclimit));
    e2.set_reset(duel, RESET_PHASE | PHASE_END);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_type(duel, EFFECT_TYPE_FIELD);
    e3.set_code(duel, EFFECT_DISABLE);
    e3.set_range(duel, LOCATION_MZONE);
    e3.set_target_range(duel, LOCATION_SZONE, LOCATION_SZONE);
    e3.set_target_card(duel, |duel, _, c| c.is_trap(duel));
    c.register_effect(duel, e3);
    let e4 = duel.create_effect(c);
    e4.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_CONTINUOUS);
    e4.set_code(duel, EVENT_CHAIN_SOLVING);
    e4.set_range(duel, LOCATION_MZONE);
    e4.set_operation(duel, disop);
    c.register_effect(duel, e4);
    let e5 = duel.create_effect(c);
    e5.set_type(duel, EFFECT_TYPE_FIELD);
    e5.set_code(duel, EFFECT_DISABLE_TRAPMONSTER);
    e5.set_range(duel, LOCATION_MZONE);
    e5.set_target_range(duel, LOCATION_MZONE, LOCATION_MZONE);
    e5.set_target_card(duel, |duel, _, c| c.is_trap(duel));
    c.register_effect(duel, e5);
    aux::double_snare_validity(duel, c, LOCATION_MZONE);
}

fn disop(duel: &mut Duel, a: &Args) -> Script<()> {
    let ev = a.ev as u8;
    let tl: u32 = duel.get_chain_info(ev, CHAININFO_TRIGGERING_LOCATION);
    if tl == LOCATION_SZONE && a.re.is_some_and(|re| re.is_trap_effect(duel)) {
        duel.negate_effect(ev);
    }
    Ok(())
}

fn aclimit(duel: &Duel, _: EffectId, re: EffectId, _: u8) -> bool {
    re.get_handler(duel).is_trap(duel)
}
