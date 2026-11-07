// Mage Power
use engine::constants::MSG_EQUIP;

use crate::prelude::*;
use crate::testing::{Check, HAND, MZONE};

const ID: u32 = 83746708;

struct Procedure;

impl EquipProcedure for Procedure {}

// In the hand, to be activated on a monster: Sangan.
pub const SUPPORT: &[(u32, u16)] = &[(ID, HAND), (26202165, MZONE)];
pub const CHECKS: &[Check] = &[Check::Message(MSG_EQUIP)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    Equip::add_procedure::<Procedure>(duel, c);
    for code in [EFFECT_UPDATE_ATTACK, EFFECT_UPDATE_DEFENSE] {
        let e = duel.create_effect(c);
        e.set_type(duel, EFFECT_TYPE_EQUIP);
        e.set_code(duel, code);
        e.set_value(duel, Value::Card(value));
        c.register_effect(duel, e);
    }
}

fn value(duel: &Duel, e: EffectId, _: CardId) -> i32 {
    duel.get_matching_group_count(CardId::is_spell_trap, e.get_handler_player(duel), LOCATION_ONFIELD, 0, None) as i32 * 500
}
