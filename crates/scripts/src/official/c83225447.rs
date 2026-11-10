// Stim-Pack
use engine::constants::MSG_EQUIP;

use crate::prelude::*;
use crate::testing::{Check, HAND, MZONE};

const ID: u32 = 83225447;

struct Procedure;

impl EquipProcedure for Procedure {}

// In the hand to equip a monster (Shapesnatch). Its other effects are continuous, with nothing a check could count.
pub const SUPPORT: &[(u32, u16)] = &[(ID, HAND), (ID, HAND), (4035199, MZONE), (4035199, MZONE)];
pub const CHECKS: &[Check] = &[Check::Message(MSG_EQUIP)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    Equip::add_procedure::<Procedure>(duel, c);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_EQUIP);
    e2.set_code(duel, EFFECT_UPDATE_ATTACK);
    e2.set_value(duel, 700);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE);
    e3.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_CONTINUOUS);
    e3.set_code(duel, EVENT_EQUIP);
    e3.set_operation(duel, regop);
    e3.set_range(duel, LOCATION_SZONE);
    e3.set_label_object(duel, Some(e2));
    c.register_effect(duel, e3);
    let e4 = duel.create_effect(c);
    e4.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_CONTINUOUS);
    e4.set_code(duel, EVENT_PHASE | PHASE_STANDBY);
    e4.set_range(duel, LOCATION_SZONE);
    e4.set_count_limit(duel, 1, 0);
    e4.set_label_object(duel, Some(e2));
    e4.set_condition(duel, |duel, a| duel.is_turn_player(a.tp));
    e4.set_operation(duel, atkop);
    c.register_effect(duel, e4);
}

fn regop(duel: &mut Duel, a: &Args) -> Script<()> {
    if !a.eg.unwrap_or_default().is_contains(a.e.get_handler(duel)) {
        return Ok(());
    }
    if let Some(pe) = a.e.get_label_object(duel) {
        pe.set_value(duel, 700);
    }
    Ok(())
}

fn atkop(duel: &mut Duel, a: &Args) -> Script<()> {
    if let Some(pe) = a.e.get_label_object(duel)
        && let Value::Int(atk) = pe.get_value(duel)
    {
        pe.set_value(duel, atk - 200);
    }
    Ok(())
}
