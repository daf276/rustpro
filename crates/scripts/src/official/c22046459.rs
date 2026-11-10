// Megamorph
use engine::constants::MSG_EQUIP;

use crate::prelude::*;
use crate::testing::{Check, HAND, MZONE};

const ID: u32 = 22046459;

struct Procedure;

impl EquipProcedure for Procedure {}

// In the hand to equip a monster (Shapesnatch).
pub const SUPPORT: &[(u32, u16)] = &[(ID, HAND), (ID, HAND), (4035199, MZONE), (4035199, MZONE)];
pub const CHECKS: &[Check] = &[Check::Message(MSG_EQUIP)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    Equip::add_procedure::<Procedure>(duel, c);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_EQUIP);
    e2.set_code(duel, EFFECT_SET_ATTACK);
    e2.set_condition(duel, |duel, _| duel.get_lp(0) != duel.get_lp(1));
    e2.set_value(duel, Value::Card(value));
    c.register_effect(duel, e2);
}

fn value(duel: &Duel, e: EffectId, c: CardId) -> i32 {
    let p = e.get_handler(duel).get_controler(duel);
    let (lp, opponent_lp) = (duel.get_lp(p), duel.get_lp(1 - p));
    if lp < opponent_lp {
        c.get_base_attack(duel) * 2
    } else if lp > opponent_lp {
        (c.get_base_attack(duel) + 1) / 2
    } else {
        0
    }
}
