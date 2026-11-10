// Heavy Mech Support Platform
use crate::prelude::*;
use crate::testing::{Check, MZONE};
use engine::constants::MSG_EQUIP;

const ID: u32 = 23265594;

struct Procedure;

impl UnionProcedure for Procedure {
    fn f(c: CardId, duel: &Duel) -> bool {
        c.is_race(duel, RACE_MACHINE)
    }
}

// On the field with a Machine monster to equip: Mechanicalchaser.
pub const SUPPORT: &[(u32, u16)] = &[(ID, MZONE), (7359741, MZONE)];
pub const CHECKS: &[Check] = &[Check::Message(MSG_EQUIP), Check::Report(&Check::Replaced)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    Union::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_EQUIP);
    e1.set_code(duel, EFFECT_UPDATE_ATTACK);
    e1.set_value(duel, 500);
    c.register_effect(duel, e1);
    let e2 = e1.clone_effect(duel);
    e2.set_code(duel, EFFECT_UPDATE_DEFENSE);
    c.register_effect(duel, e2);
}
