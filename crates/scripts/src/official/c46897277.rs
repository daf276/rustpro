// Wattfox
use crate::prelude::*;
use crate::testing::{HAND, MZONE, OPPONENT};

const ID: u32 = 46897277;

// Destroyed by the opponent's Raigeki.
pub const SUPPORT: &[(u32, u16)] = &[(ID, MZONE), (ID, MZONE), (12580477, HAND | OPPONENT)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_F);
    e1.set_code(duel, EVENT_DESTROYED);
    e1.set_condition(duel, condition);
    e1.set_operation(duel, operation);
    c.register_effect(duel, e1);
}

fn condition(duel: &Duel, a: &Args) -> bool {
    a.rp == 1 - a.tp && a.e.get_handler(duel).get_previous_controler(duel) == a.tp
}

fn operation(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    let c = a.e.get_handler(duel);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_property(duel, EFFECT_FLAG_PLAYER_TARGET);
    e1.set_code(duel, EFFECT_CANNOT_ACTIVATE);
    e1.set_target_range(duel, 0, 1);
    e1.set_value(duel, 1);
    e1.set_reset(duel, RESET_PHASE | PHASE_END);
    duel.register_effect(e1, tp);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_FIELD);
    e2.set_property(duel, EFFECT_FLAG_PLAYER_TARGET);
    e2.set_code(duel, EFFECT_CANNOT_SPECIAL_SUMMON);
    e2.set_reset(duel, RESET_PHASE | PHASE_END);
    e2.set_target_range(duel, 0, 1);
    duel.register_effect(e2, tp);
    Ok(())
}
