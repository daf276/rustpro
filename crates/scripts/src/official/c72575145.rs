// Demotion
use engine::constants::MSG_EQUIP;

use crate::prelude::*;
use crate::testing::{Check, HAND};

struct Procedure;

impl EquipProcedure for Procedure {}

// In the hand, to be activated on a face-up monster.
pub const SUPPORT: &[(u32, u16)] = &[(72575145, HAND)];
pub const CHECKS: &[Check] = &[Check::Message(MSG_EQUIP)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    Equip::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_EQUIP);
    e1.set_code(duel, EFFECT_UPDATE_LEVEL);
    e1.set_value(duel, -2);
    c.register_effect(duel, e1);
}
