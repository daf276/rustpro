// Y-Dragon Head
use engine::constants::MSG_EQUIP;

use crate::prelude::*;
use crate::testing::{Check, MZONE};

const ID: u32 = 65622692;
const X_HEAD_CANNON: u32 = 62651957;

struct Procedure;

impl UnionProcedure for Procedure {
    fn f(c: CardId, duel: &Duel) -> bool {
        c.is_code(duel, X_HEAD_CANNON)
    }
}

pub const LISTED_NAMES: &[u32] = &[X_HEAD_CANNON];

// On the field with X-Head Cannon to equip to.
pub const SUPPORT: &[(u32, u16)] = &[(ID, MZONE), (X_HEAD_CANNON, MZONE)];
pub const CHECKS: &[Check] = &[Check::Message(MSG_EQUIP), Check::Report(&Check::Replaced)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    Union::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_EQUIP);
    e1.set_code(duel, EFFECT_UPDATE_ATTACK);
    e1.set_value(duel, 400);
    c.register_effect(duel, e1);
    let e2 = e1.clone_effect(duel);
    e2.set_code(duel, EFFECT_UPDATE_DEFENSE);
    c.register_effect(duel, e2);
}
