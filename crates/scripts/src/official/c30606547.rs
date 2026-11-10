// The Dark Door
use engine::constants::MSG_ATTACK;

use crate::prelude::*;
use crate::testing::{Check, HAND, MZONE, OPPONENT};

const ID: u32 = 30606547;

// In the hand to be activated, with Elfnote Lucina (13597785) on both sides to attack.
pub const SUPPORT: &[(u32, u16)] = &[(ID, HAND), (13597785, MZONE), (13597785, MZONE | OPPONENT)];
pub const CHECKS: &[Check] = &[Check::Resolves, Check::Message(MSG_ATTACK)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_FIELD);
    e2.set_property(duel, EFFECT_FLAG_IGNORE_IMMUNE);
    e2.set_code(duel, EFFECT_CANNOT_ATTACK_ANNOUNCE);
    e2.set_range(duel, LOCATION_SZONE);
    e2.set_target_range(duel, LOCATION_MZONE, LOCATION_MZONE);
    e2.set_condition(duel, |duel, a| a.e.get_handler(duel).has_flag_effect(duel, ID));
    e2.set_target_card(duel, |duel, e, c| i64::from(c.get_field_id(duel)) != e.get_label(duel));
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_CONTINUOUS);
    e3.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE);
    e3.set_code(duel, EVENT_ATTACK_ANNOUNCE);
    e3.set_range(duel, LOCATION_SZONE);
    e3.set_operation(duel, checkop);
    e3.set_label_object(duel, Some(e2));
    c.register_effect(duel, e3);
}

fn checkop(duel: &mut Duel, a: &Args) -> Script<()> {
    let c = a.e.get_handler(duel);
    if c.has_flag_effect(duel, ID) {
        return Ok(());
    }
    let fid = a.eg.and_then(|eg| eg.get_first()).map_or(0, |tc| tc.get_field_id(duel));
    c.register_flag_effect(duel, ID, RESETS_STANDARD_PHASE_END, 0, 1);
    if let Some(e2) = a.e.get_label_object(duel) {
        e2.set_label(duel, i64::from(fid));
    }
    Ok(())
}
