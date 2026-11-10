// Chain Energy
use crate::prelude::*;
use crate::testing::{Check, HAND};
use engine::constants::MSG_PAY_LPCOST;

const ID: u32 = 79323590;

// Two copies in the hand, so that a second one pays for the first one's accumulated cost.
pub const SUPPORT: &[(u32, u16)] = &[(ID, HAND), (ID, HAND)];
pub const CHECKS: &[Check] = &[Check::Resolves, Check::Message(MSG_PAY_LPCOST)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_FIELD);
    e2.set_code(duel, EFFECT_ACTIVATE_COST);
    e2.set_range(duel, LOCATION_SZONE);
    e2.set_property(duel, EFFECT_FLAG_PLAYER_TARGET);
    e2.set_target_range(duel, 1, 1);
    e2.set_target_activate_cost(duel, |duel, _, te, _| te.get_handler(duel).is_location(duel, LOCATION_HAND));
    e2.set_cost_activate(duel, |duel, _, _, tp| costchk(duel, tp));
    e2.set_operation(duel, costop);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_type(duel, EFFECT_TYPE_FIELD);
    e3.set_code(duel, EFFECT_SUMMON_COST);
    e3.set_range(duel, LOCATION_SZONE);
    e3.set_target_range(duel, LOCATION_HAND, LOCATION_HAND);
    e3.set_cost_condition(duel, |duel, _, _, tp, _| costchk(duel, tp));
    e3.set_operation(duel, costop);
    c.register_effect(duel, e3);
    for code in [EFFECT_SPSUMMON_COST, EFFECT_MSET_COST, EFFECT_SSET_COST] {
        let e = e3.clone_effect(duel);
        e.set_code(duel, code);
        c.register_effect(duel, e);
    }
    let e7 = duel.create_effect(c);
    e7.set_type(duel, EFFECT_TYPE_FIELD);
    e7.set_code(duel, ID);
    e7.set_range(duel, LOCATION_SZONE);
    e7.set_property(duel, EFFECT_FLAG_PLAYER_TARGET);
    e7.set_target_range(duel, 1, 1);
    c.register_effect(duel, e7);
}

fn costchk(duel: &Duel, tp: u8) -> bool {
    duel.check_lp_cost(tp, duel.get_player_effect(tp, ID).len() as u32 * 500)
}

fn costop(duel: &mut Duel, a: &Args) -> Script<()> {
    duel.hint(HINT_CARD, 0, ID as u64);
    duel.pay_lp_cost(a.tp, 500)
}
