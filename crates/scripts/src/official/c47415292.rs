// Pitch-Dark Dragon
use crate::prelude::*;
use crate::testing::{Check, MZONE};

const ID: u32 = 47415292;
const DARK_BLADE: u32 = 11321183;

struct Procedure;

impl UnionProcedure for Procedure {
    const OLD_EQUIP: bool = true;

    fn f(c: CardId, duel: &Duel) -> bool {
        c.is_code(duel, DARK_BLADE)
    }
}

pub const LISTED_NAMES: &[u32] = &[DARK_BLADE];
// On the field with Dark Blade (a Normal Monster) to equip.
pub const SUPPORT: &[(u32, u16)] = &[(ID, MZONE), (DARK_BLADE, MZONE)];
// Piercing damage and the Union's destruction substitution are too rare to assert.
pub const CHECKS: &[Check] = &[Check::Resolves, Check::Report(&Check::Replaced)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    Union::add_procedure::<Procedure>(duel, c);
    let e3 = duel.create_effect(c);
    e3.set_type(duel, EFFECT_TYPE_EQUIP);
    e3.set_code(duel, EFFECT_UPDATE_ATTACK);
    e3.set_value(duel, 400);
    e3.set_condition(duel, Union::is_union_state);
    c.register_effect(duel, e3);
    let e4 = e3.clone_effect(duel);
    e4.set_code(duel, EFFECT_UPDATE_DEFENSE);
    c.register_effect(duel, e4);
    let e5 = duel.create_effect(c);
    e5.set_type(duel, EFFECT_TYPE_EQUIP);
    e5.set_code(duel, EFFECT_PIERCE);
    e5.set_condition(duel, Union::is_union_state);
    c.register_effect(duel, e5);
}
