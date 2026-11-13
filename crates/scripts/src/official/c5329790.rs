// Protocol Gardna
use super::c49847524::Procedure;
use crate::prelude::*;
use crate::testing::{Check, EXTRA, MZONE, OPPONENT};
use engine::constants::MSG_ATTACK;

const ID: u32 = 5329790;

// In the Extra Deck, Link Summoned from Cyberse Normal Monsters, then attacked by the opponent's Elfnote Lucina.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (24154052, MZONE),
    (32295838, MZONE),
    (24154052, MZONE),
    (13597785, MZONE | OPPONENT),
    (13597785, MZONE | OPPONENT),
];
// All of its effects are continuous.
pub const CHECKS: &[Check] = &[Check::Message(MSG_ATTACK)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    Link::add_procedure::<Procedure>(duel, c);
    c.enable_revive_limit(duel);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_code(duel, EFFECT_CANNOT_SELECT_BATTLE_TARGET);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_target_range(duel, 0, LOCATION_MZONE);
    e1.set_value(
        duel,
        Value::Card(|duel, e, c| i32::from(e.get_handler(duel).get_linked_group(duel).is_contains(c))),
    );
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_SINGLE);
    e2.set_property(duel, EFFECT_FLAG_SINGLE_RANGE);
    e2.set_code(duel, EFFECT_INDESTRUCTABLE_COUNT);
    e2.set_count_limit(duel, 1, 0);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_value(duel, Value::ReasonMut(valcon));
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_type(duel, EFFECT_TYPE_SINGLE);
    e3.set_code(duel, EFFECT_AVOID_BATTLE_DAMAGE);
    e3.set_value(
        duel,
        Value::Card(|duel, e, _| i32::from(e.get_handler(duel).get_flag_effect(duel, ID) == 0)),
    );
    c.register_effect(duel, e3);
}

fn valcon(duel: &mut Duel, e: EffectId, _: Option<EffectId>, r: u32, _: u8) -> bool {
    if r & REASON_BATTLE != 0 {
        e.get_handler(duel).register_flag_effect(duel, ID, RESETS_STANDARD_PHASE_END, 0, 1);
        return true;
    }
    false
}
