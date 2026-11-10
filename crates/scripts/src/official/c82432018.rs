// Mask of Brutality
use engine::constants::{MSG_EQUIP, MSG_PAY_LPCOST};

use crate::prelude::*;
use crate::testing::{Check, HAND, MZONE};

const ID: u32 = 82432018;

struct Procedure;

impl EquipProcedure for Procedure {}

// In the hand, to be activated on a monster: Eldeen.
pub const SUPPORT: &[(u32, u16)] = &[(ID, HAND), (6367785, MZONE)];
pub const CHECKS: &[Check] = &[Check::Message(MSG_EQUIP), Check::Message(MSG_PAY_LPCOST)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    Equip::add_procedure::<Procedure>(duel, c);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_EQUIP);
    e2.set_code(duel, EFFECT_UPDATE_ATTACK);
    e2.set_value(duel, 1000);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_type(duel, EFFECT_TYPE_EQUIP);
    e3.set_code(duel, EFFECT_UPDATE_DEFENSE);
    e3.set_value(duel, -1000);
    c.register_effect(duel, e3);
    let e5 = duel.create_effect(c);
    e5.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_CONTINUOUS);
    e5.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE + EFFECT_FLAG_UNCOPYABLE);
    e5.set_code(duel, EVENT_PHASE | PHASE_STANDBY);
    e5.set_range(duel, LOCATION_SZONE);
    e5.set_count_limit(duel, 1, 0);
    e5.set_condition(duel, |duel, a| duel.is_turn_player(a.tp));
    e5.set_operation(duel, mtop);
    c.register_effect(duel, e5);
}

fn mtop(duel: &mut Duel, a: &Args) -> Script<()> {
    if duel.check_lp_cost(a.tp, 1000) && duel.select_yes_no(a.tp, aux::stringid(ID, 0))? {
        duel.pay_lp_cost(a.tp, 1000)?;
    } else {
        duel.destroy(a.e.get_handler(duel), REASON_COST)?;
    }
    Ok(())
}
