// Fatal Abacus
use engine::constants::MSG_DAMAGE;

use crate::prelude::*;
use crate::testing::{Check, HAND, MZONE, OPPONENT};

const ID: u32 = 77910045;

// In the hand, with monsters on both sides for the opponent's Raigeki to destroy.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, HAND),
    (756652, MZONE),
    (756652, MZONE | OPPONENT),
    (756652, MZONE | OPPONENT),
    (12580477, HAND | OPPONENT),
    (12580477, HAND | OPPONENT),
];
pub const CHECKS: &[Check] = &[Check::Message(MSG_DAMAGE)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_CONTINUOUS);
    e2.set_code(duel, EVENT_TO_GRAVE);
    e2.set_range(duel, LOCATION_SZONE);
    e2.set_operation(duel, operation);
    c.register_effect(duel, e2);
}

fn filter(c: CardId, duel: &Duel, tp: u8) -> bool {
    c.is_previous_location(duel, LOCATION_MZONE) && c.is_controler(duel, tp) && c.is_monster(duel)
}

fn operation(duel: &mut Duel, a: &Args) -> Script<()> {
    let eg = a.eg.unwrap_or_default();
    for p in [a.tp, 1 - a.tp] {
        let ct = eg.filter_count(duel, |c, duel| filter(c, duel, p), None);
        if ct > 0 {
            duel.damage(p, 500 * ct as u32, REASON_EFFECT)?;
        }
    }
    Ok(())
}
