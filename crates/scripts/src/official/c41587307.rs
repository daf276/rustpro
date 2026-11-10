// Broken Bamboo Sword
use engine::constants::MSG_EQUIP;

use crate::prelude::*;
use crate::testing::{Check, HAND, MZONE};

const ID: u32 = 41587307;

struct Procedure;

impl EquipProcedure for Procedure {}

// In the hand to equip a monster (Shapesnatch).
pub const SUPPORT: &[(u32, u16)] = &[(ID, HAND), (ID, HAND), (4035199, MZONE), (4035199, MZONE)];
pub const CHECKS: &[Check] = &[Check::Message(MSG_EQUIP)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    Equip::add_procedure::<Procedure>(duel, c);
    let e0 = duel.create_effect(c);
    e0.set_type(duel, EFFECT_TYPE_EQUIP);
    e0.set_code(duel, EFFECT_UPDATE_ATTACK);
    e0.set_value(duel, 0);
    c.register_effect(duel, e0);
}
