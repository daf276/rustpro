// Electro-Whip
use engine::constants::MSG_EQUIP;

use crate::prelude::*;
use crate::testing::{Check, HAND, MZONE};

const ID: u32 = 37820550;

struct Procedure;

impl EquipProcedure for Procedure {
    fn f(c: CardId, duel: &Duel, _: EffectId, _: u8) -> bool {
        c.is_race(duel, RACE_THUNDER)
    }
}

// In the hand, to be activated on a Thunder: Surge Blitzclique.
pub const SUPPORT: &[(u32, u16)] = &[(ID, HAND), (22912101, MZONE)];
pub const CHECKS: &[Check] = &[Check::Message(MSG_EQUIP)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    Equip::add_procedure::<Procedure>(duel, c);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_EQUIP);
    e2.set_code(duel, EFFECT_UPDATE_ATTACK);
    e2.set_value(duel, 300);
    c.register_effect(duel, e2);
    let e3 = e2.clone_effect(duel);
    e3.set_code(duel, EFFECT_UPDATE_DEFENSE);
    c.register_effect(duel, e3);
}
