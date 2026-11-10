// Mystical Moon
use engine::constants::MSG_EQUIP;

use crate::prelude::*;
use crate::testing::{Check, HAND, MZONE};

const ID: u32 = 36607978;

struct Procedure;

impl EquipProcedure for Procedure {
    fn f(c: CardId, duel: &Duel, _: EffectId, _: u8) -> bool {
        c.is_race(duel, RACE_BEASTWARRIOR)
    }
}

// In the hand, to be activated on a Beast-Warrior: Battle Ox.
pub const SUPPORT: &[(u32, u16)] = &[(ID, HAND), (5053103, MZONE)];
pub const CHECKS: &[Check] = &[Check::Message(MSG_EQUIP)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    Equip::add_procedure::<Procedure>(duel, c);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_EQUIP);
    e2.set_code(duel, EFFECT_UPDATE_ATTACK);
    e2.set_value(duel, 300);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_type(duel, EFFECT_TYPE_EQUIP);
    e3.set_code(duel, EFFECT_UPDATE_DEFENSE);
    e3.set_value(duel, 300);
    c.register_effect(duel, e3);
}
