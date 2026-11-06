// Demotion
use crate::prelude::*;

struct Procedure;

impl EquipProcedure for Procedure {}

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    Equip::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_EQUIP);
    e1.set_code(duel, EFFECT_UPDATE_LEVEL);
    e1.set_value(duel, -2);
    c.register_effect(duel, e1);
}
