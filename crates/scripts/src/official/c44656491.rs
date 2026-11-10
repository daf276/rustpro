// Messenger of Peace
use engine::constants::MSG_SELECT_YESNO;

use crate::prelude::*;
use crate::testing::{Check, HAND, MZONE, OPPONENT};

const ID: u32 = 44656491;

// In the hand to Activate, with monsters of 1500 ATK or more on both fields (Summoned Skull, Cyber-Tech Alligator).
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (70781052, MZONE),
    (48766543, MZONE | OPPONENT),
    (70781052, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[Check::Resolves, Check::Message(MSG_SELECT_YESNO)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_FIELD);
    e2.set_code(duel, EFFECT_CANNOT_ATTACK_ANNOUNCE);
    e2.set_range(duel, LOCATION_SZONE);
    e2.set_target_range(duel, LOCATION_MZONE, LOCATION_MZONE);
    e2.set_target_card(duel, |duel, _, c| c.get_attack(duel) >= 1500);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_CONTINUOUS);
    e3.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE + EFFECT_FLAG_UNCOPYABLE);
    e3.set_code(duel, EVENT_PHASE + PHASE_STANDBY);
    e3.set_range(duel, LOCATION_SZONE);
    e3.set_count_limit(duel, 1, 0);
    e3.set_condition(duel, |duel, a| duel.is_turn_player(a.tp));
    e3.set_operation(duel, mtop);
    c.register_effect(duel, e3);
}

fn mtop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    if duel.check_lp_cost(tp, 100) && duel.select_yes_no(tp, aux::stringid(ID, 0))? {
        duel.pay_lp_cost(tp, 100)?;
    } else {
        duel.destroy(a.e.get_handler(duel), REASON_COST)?;
    }
    Ok(())
}
