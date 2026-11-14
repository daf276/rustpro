// Vector Pendulum, the Dracoverlord
use crate::prelude::*;
use crate::testing::{Check, HAND, OPPONENT, PZONE};
use engine::constants::MSG_CHAIN_DISABLED;

const ID: u32 = 69512157;

// In the Pendulum Zone, with Pendulum Monsters in the opponent's hand to activate.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, PZONE),
    (17390179, HAND | OPPONENT),
    (17390179, HAND | OPPONENT),
    (17390179, HAND | OPPONENT),
    (77994337, HAND | OPPONENT),
    (77994337, HAND | OPPONENT),
];
pub const CHECKS: &[Check] = &[Check::Message(MSG_CHAIN_DISABLED)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    Pendulum::add_procedure(duel, c);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_FIELD);
    e2.set_code(duel, EFFECT_DISABLE);
    e2.set_range(duel, LOCATION_PZONE);
    e2.set_target_range(duel, 0, LOCATION_PZONE);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_CONTINUOUS);
    e3.set_code(duel, EVENT_CHAIN_SOLVING);
    e3.set_range(duel, LOCATION_PZONE);
    e3.set_operation(duel, disop);
    c.register_effect(duel, e3);
}

fn disop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (tp, ev) = (a.tp, a.ev as u8);
    let p: u8 = duel.get_chain_info(ev, CHAININFO_TRIGGERING_CONTROLER);
    let loc: u32 = duel.get_chain_info(ev, CHAININFO_TRIGGERING_LOCATION);
    if let Some(re) = a.re
        && re.get_active_type(duel) == TYPE_PENDULUM + TYPE_SPELL
        && p != tp
        && loc & LOCATION_PZONE != 0
    {
        duel.negate_effect(ev);
    }
    Ok(())
}
