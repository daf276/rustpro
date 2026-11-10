// Lightning Blade
use engine::constants::MSG_EQUIP;

use crate::prelude::*;
use crate::testing::{Check, HAND, MZONE, OPPONENT};

const ID: u32 = 55226821;

struct Procedure;

impl EquipProcedure for Procedure {
    fn f(c: CardId, duel: &Duel, _: EffectId, _: u8) -> bool {
        c.is_race(duel, RACE_WARRIOR)
    }
}

// In the hand, to be activated on a Warrior (Doron) on either side, with WATER monsters (Psychic Kappa) for the Field effect.
pub const SUPPORT: &[(u32, u16)] = &[(ID, HAND), (756652, MZONE), (756652, MZONE | OPPONENT), (7892180, MZONE | OPPONENT)];
pub const CHECKS: &[Check] = &[Check::Message(MSG_EQUIP)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    Equip::add_procedure::<Procedure>(duel, c);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_EQUIP);
    e2.set_code(duel, EFFECT_UPDATE_ATTACK);
    e2.set_value(duel, 800);
    c.register_effect(duel, e2);
    let e4 = duel.create_effect(c);
    e4.set_type(duel, EFFECT_TYPE_FIELD);
    e4.set_range(duel, LOCATION_SZONE);
    e4.set_target_range(duel, LOCATION_MZONE, LOCATION_MZONE);
    e4.set_code(duel, EFFECT_UPDATE_ATTACK);
    e4.set_target_card(duel, |duel, _, c| c.is_attribute(duel, ATTRIBUTE_WATER));
    e4.set_value(duel, -500);
    c.register_effect(duel, e4);
}
