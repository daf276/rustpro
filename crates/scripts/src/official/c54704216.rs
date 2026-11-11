// Nightmare Wheel
use engine::constants::MSG_CARD_TARGET;

use crate::prelude::*;
use crate::testing::{Check, MZONE, OPPONENT, SZONE};

const ID: u32 = 54704216;

// Set, to be activated on the opponent's Effect Monster Elfnote Fortuna.
pub const SUPPORT: &[(u32, u16)] = &[(ID, SZONE), (ID, SZONE), (85976588, MZONE | OPPONENT)];
pub const CHECKS: &[Check] = &[Check::Message(MSG_CARD_TARGET), Check::ResolvesEffect(aux::stringid(ID, 0))];

struct Procedure;

impl PersistentProcedure for Procedure {
    const P: Option<u8> = Some(1);
    const CATEGORY: u64 = CATEGORY_POSITION;
    const HINT1: Option<u32> = Some(TIMING_STANDBY_PHASE);
    const HINT2: Option<u32> = Some(TIMINGS_CHECK_MONSTER);
    const ANYPOS: bool = true;
}

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    Persistent::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_code(duel, EFFECT_CANNOT_ATTACK);
    e1.set_range(duel, LOCATION_SZONE);
    e1.set_target_range(duel, LOCATION_MZONE, LOCATION_MZONE);
    e1.set_target_card(duel, Persistent::target_filter);
    c.register_effect(duel, e1);
    let e2 = e1.clone_effect(duel);
    e2.set_code(duel, EFFECT_CANNOT_CHANGE_POSITION);
    e2.set_property(duel, EFFECT_FLAG_SET_AVAILABLE);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 0));
    e3.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_F);
    e3.set_category(duel, CATEGORY_DAMAGE);
    e3.set_code(duel, EVENT_PHASE | PHASE_STANDBY);
    e3.set_range(duel, LOCATION_SZONE);
    e3.set_count_limit(duel, 1, 0);
    e3.set_condition(duel, |duel, a| {
        duel.is_turn_player(a.tp) && a.e.get_handler(duel).get_first_card_target(duel).is_some()
    });
    e3.set_target(duel, damtg);
    e3.set_operation(duel, stock::damage);
    c.register_effect(duel, e3);
    let e4 = duel.create_effect(c);
    e4.set_type(duel, EFFECT_TYPE_CONTINUOUS + EFFECT_TYPE_FIELD);
    e4.set_range(duel, LOCATION_SZONE);
    e4.set_code(duel, EVENT_LEAVE_FIELD);
    e4.set_condition(duel, super::c25542642::descon);
    e4.set_operation(duel, super::c25542642::desop);
    c.register_effect(duel, e4);
}

fn damtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(true);
    }
    duel.set_target_player(1 - tp);
    duel.set_target_param(500);
    duel.set_operation_info(0, CATEGORY_DAMAGE, None, 0, 1 - tp, 500);
    Ok(false)
}
