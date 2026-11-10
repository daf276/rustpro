// United We Stand
use engine::constants::MSG_EQUIP;

use crate::prelude::*;
use crate::testing::{Check, HAND, MZONE};

const ID: u32 = 56747793;

struct Procedure;

impl EquipProcedure for Procedure {}

// In the hand to equip a monster (Shapesnatch), with more monsters on the field to count.
pub const SUPPORT: &[(u32, u16)] = &[(ID, HAND), (ID, HAND), (4035199, MZONE), (4035199, MZONE)];
pub const CHECKS: &[Check] = &[Check::Message(MSG_EQUIP)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    Equip::add_procedure::<Procedure>(duel, c);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_EQUIP);
    e2.set_code(duel, EFFECT_UPDATE_ATTACK);
    e2.set_value(duel, Value::Card(value));
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_type(duel, EFFECT_TYPE_EQUIP);
    e3.set_code(duel, EFFECT_UPDATE_DEFENSE);
    e3.set_value(duel, Value::Card(value));
    c.register_effect(duel, e3);
}

fn value(duel: &Duel, e: EffectId, _: CardId) -> i32 {
    let controler = e.get_handler(duel).get_controler(duel);
    duel.get_matching_group_count(CardId::is_faceup, controler, LOCATION_MZONE, 0, None) as i32 * 800
}
