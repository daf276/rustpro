// Insect Soldiers of the Sky
use crate::prelude::*;
use crate::testing::{Check, MZONE, OPPONENT};
use engine::constants::MSG_ATTACK;

const ID: u32 = 7019529;

// Two on the field to attack the opponent's WIND Luster Dragons; a continuous effect, so the attacks are counted.
pub const SUPPORT: &[(u32, u16)] = &[(ID, MZONE), (ID, MZONE), (11091375, MZONE | OPPONENT), (11091375, MZONE | OPPONENT)];
pub const CHECKS: &[Check] = &[Check::Message(MSG_ATTACK)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_property(duel, EFFECT_FLAG_SINGLE_RANGE);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_code(duel, EFFECT_UPDATE_ATTACK);
    e1.set_condition(duel, condtion);
    e1.set_value(duel, 1000);
    c.register_effect(duel, e1);
}

fn condtion(duel: &Duel, a: &Args) -> bool {
    let ph = duel.get_current_phase();
    (ph == PHASE_DAMAGE || ph == PHASE_DAMAGE_CAL)
        && duel.get_attacker() == Some(a.e.get_handler(duel))
        && duel
            .get_attack_target()
            .is_some_and(|t| t.is_faceup(duel) && t.is_attribute(duel, ATTRIBUTE_WIND))
}
