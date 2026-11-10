// Solemn Wishes
use engine::constants::MSG_RECOVER;

use crate::prelude::*;
use crate::testing::{Check, HAND, SZONE};

const ID: u32 = 35346968;

// In the hand to be activated, and Set to be activated before the next draw.
pub const SUPPORT: &[(u32, u16)] = &[(ID, HAND), (ID, SZONE)];
pub const CHECKS: &[Check] = &[Check::Resolves, Check::Message(MSG_RECOVER)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_hint_timing(duel, 0, TIMING_END_PHASE);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_CONTINUOUS);
    e2.set_property(duel, EFFECT_FLAG_DELAY);
    e2.set_range(duel, LOCATION_SZONE);
    e2.set_code(duel, EVENT_DRAW);
    e2.set_operation(duel, recop);
    c.register_effect(duel, e2);
}

fn recop(duel: &mut Duel, a: &Args) -> Script<()> {
    if a.ep != a.tp {
        return Ok(());
    }
    duel.recover(a.tp, 500, REASON_EFFECT)?;
    Ok(())
}
