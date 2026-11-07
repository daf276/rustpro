// Salamandra
use engine::constants::MSG_EQUIP;

use crate::prelude::*;
use crate::testing::{Check, HAND, MZONE};

struct Procedure;

impl EquipProcedure for Procedure {
    fn f(c: CardId, duel: &Duel, _: EffectId, _: u8) -> bool {
        c.is_attribute(duel, ATTRIBUTE_FIRE)
    }
}

// In the hand, to be activated on a FIRE monster: Elfnote Lucina.
pub const SUPPORT: &[(u32, u16)] = &[(32268901, HAND), (13597785, MZONE)];
pub const CHECKS: &[Check] = &[Check::Message(MSG_EQUIP)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    Equip::add_procedure::<Procedure>(duel, c);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_EQUIP);
    e2.set_code(duel, EFFECT_UPDATE_ATTACK);
    e2.set_value(duel, 700);
    c.register_effect(duel, e2);
}
