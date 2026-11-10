// Mirage Dragon
use crate::prelude::*;
use crate::testing::{Check, MZONE, OPPONENT, SZONE};
use engine::constants::MSG_ATTACK;

const ID: u32 = 15960641;

// On the field, with Negate Attack Set for the opponent to activate in its battle phase.
pub const SUPPORT: &[(u32, u16)] = &[(ID, MZONE), (14315573, SZONE | OPPONENT), (14315573, SZONE | OPPONENT)];
pub const CHECKS: &[Check] = &[Check::Message(MSG_ATTACK)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_property(duel, EFFECT_FLAG_PLAYER_TARGET);
    e1.set_code(duel, EFFECT_CANNOT_ACTIVATE);
    e1.set_condition(duel, |duel, _| duel.is_battle_phase());
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_target_range(duel, 0, 1);
    e1.set_value(duel, Value::EffectPlayer(aclimit));
    c.register_effect(duel, e1);
}

fn aclimit(duel: &Duel, _: EffectId, re: EffectId, _: u8) -> bool {
    re.get_handler(duel).is_trap(duel) && re.is_has_type(duel, EFFECT_TYPE_ACTIVATE)
}
