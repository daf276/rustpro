// Traptrix Atrax
use crate::prelude::*;
use crate::testing::{Check, MZONE, OPPONENT};
use engine::constants::MSG_ATTACK;

const ID: u32 = 55428242;

// Continuous effects only (no "Hole" Normal Trap is ported); on the field to attack and be attacked.
pub const SUPPORT: &[(u32, u16)] = &[(ID, MZONE), (ID, MZONE), (26202165, MZONE | OPPONENT), (13597785, MZONE | OPPONENT)];
pub const CHECKS: &[Check] = &[Check::Message(MSG_ATTACK)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_property(duel, EFFECT_FLAG_SINGLE_RANGE);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_code(duel, EFFECT_IMMUNE_EFFECT);
    e1.set_value(duel, Value::Effect(|duel, _, te| is_normal_hole(te.get_handler(duel), duel)));
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 0));
    e2.set_type(duel, EFFECT_TYPE_FIELD);
    e2.set_code(duel, EFFECT_TRAP_ACT_IN_HAND);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_target_range(duel, LOCATION_HAND, 0);
    e2.set_target_card(duel, |duel, _, c| is_normal_hole(c, duel));
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_type(duel, EFFECT_TYPE_FIELD);
    e3.set_code(duel, EFFECT_CANNOT_INACTIVATE);
    e3.set_range(duel, LOCATION_MZONE);
    e3.set_value(duel, Value::Chain(chainfilter));
    c.register_effect(duel, e3);
    let e4 = duel.create_effect(c);
    e4.set_type(duel, EFFECT_TYPE_FIELD);
    e4.set_code(duel, EFFECT_CANNOT_DISEFFECT);
    e4.set_range(duel, LOCATION_MZONE);
    e4.set_value(duel, Value::Chain(chainfilter));
    c.register_effect(duel, e4);
    let e5 = duel.create_effect(c);
    e5.set_type(duel, EFFECT_TYPE_FIELD);
    e5.set_code(duel, EFFECT_CANNOT_DISABLE);
    e5.set_range(duel, LOCATION_MZONE);
    e5.set_target_range(duel, LOCATION_ONFIELD, 0);
    e5.set_target_card(duel, |duel, _, c| {
        c.is_normal_trap(duel) || c.is_composite_type(duel, TYPE_TRAP | TYPE_EQUIP)
    });
    c.register_effect(duel, e5);
}

fn is_normal_hole(c: CardId, duel: &Duel) -> bool {
    c.is_normal_trap(duel) && c.is_set_card(duel, &[SET_TRAP_HOLE, SET_HOLE])
}

fn chainfilter(duel: &Duel, e: EffectId, ct: u8) -> bool {
    chain::is_triggering_controler(duel, ct, e.get_handler_player(duel)) && chain::is_triggering_exact_type(duel, ct, TYPE_TRAP)
}
