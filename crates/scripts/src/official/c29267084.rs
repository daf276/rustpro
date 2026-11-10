// Shadow Spell
use engine::ConditionFn;
use engine::constants::MSG_CARD_TARGET;

use super::c25542642::{descon, desop};
use crate::prelude::*;
use crate::testing::{Check, MZONE, OPPONENT, SZONE};

struct Procedure;

impl PersistentProcedure for Procedure {
    const P: Option<u8> = Some(1);
    const CATEGORY: u64 = CATEGORY_POSITION;
    const PROPERTY: u32 = EFFECT_FLAG_DAMAGE_STEP;
    const HINT1: Option<u32> = Some(TIMING_DAMAGE_STEP);
    const HINT2: Option<u32> = Some(TIMING_DAMAGE_STEP);
    const CON: Option<ConditionFn> = Some(aux::stat_change_damage_step_condition);

    fn f(c: CardId, duel: &Duel, _: EffectId, _: u8) -> bool {
        c.is_faceup(duel)
    }
}

// Set, to be activated on the opponent's Effect Monster Elfnote Fortuna.
pub const SUPPORT: &[(u32, u16)] = &[(29267084, SZONE), (85976588, MZONE | OPPONENT)];
pub const CHECKS: &[Check] = &[Check::Message(MSG_CARD_TARGET)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    Persistent::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_code(duel, EFFECT_CANNOT_CHANGE_POSITION);
    e1.set_range(duel, LOCATION_SZONE);
    e1.set_target_range(duel, LOCATION_MZONE, LOCATION_MZONE);
    e1.set_target_card(duel, Persistent::target_filter);
    c.register_effect(duel, e1);
    let e2 = e1.clone_effect(duel);
    e2.set_code(duel, EFFECT_CANNOT_ATTACK);
    c.register_effect(duel, e2);
    let e3 = e1.clone_effect(duel);
    e3.set_code(duel, EFFECT_UPDATE_ATTACK);
    e3.set_value(duel, -700);
    c.register_effect(duel, e3);
    let e4 = duel.create_effect(c);
    e4.set_type(duel, EFFECT_TYPE_CONTINUOUS + EFFECT_TYPE_FIELD);
    e4.set_range(duel, LOCATION_SZONE);
    e4.set_code(duel, EVENT_LEAVE_FIELD);
    e4.set_condition(duel, descon);
    e4.set_operation(duel, desop);
    c.register_effect(duel, e4);
}
