// Torque Tune Gear
use crate::prelude::*;
use crate::testing::MZONE;

const ID: u32 = 79538761;

struct Procedure;

impl UnionProcedure for Procedure {}

// On the field with monsters to equip.
pub const SUPPORT: &[(u32, u16)] = &[(ID, MZONE), (48531733, MZONE), (5628232, MZONE)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    Union::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_EQUIP);
    e1.set_code(duel, EFFECT_ADD_TYPE);
    e1.set_value(duel, TYPE_TUNER as i32);
    c.register_effect(duel, e1);
    let e2 = e1.clone_effect(duel);
    e2.set_code(duel, EFFECT_UPDATE_ATTACK);
    e2.set_value(duel, 500);
    c.register_effect(duel, e2);
    let e3 = e2.clone_effect(duel);
    e3.set_code(duel, EFFECT_UPDATE_DEFENSE);
    c.register_effect(duel, e3);
}
