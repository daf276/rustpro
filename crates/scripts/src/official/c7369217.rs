// Metallizing Parasite - Lunatite
use crate::prelude::*;
use crate::testing::{Check, MZONE};

struct Procedure;

impl UnionProcedure for Procedure {
    const OLD_EQUIP: bool = true;
}

// On the field with a monster to equip: Elfnote Lucina.
pub const SUPPORT: &[(u32, u16)] = &[(7369217, MZONE), (13597785, MZONE)];
// Destruction substitutions are too rare to assert.
pub const CHECKS: &[Check] = &[Check::Resolves, Check::Report(&Check::Replaced)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    Union::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_EQUIP);
    e1.set_code(duel, EFFECT_IMMUNE_EFFECT);
    e1.set_condition(duel, Union::is_union_state);
    e1.set_value(
        duel,
        Value::Effect(|duel, e, te| te.get_owner_player(duel) != e.get_handler_player(duel) && te.is_spell_effect(duel)),
    );
    c.register_effect(duel, e1);
}
