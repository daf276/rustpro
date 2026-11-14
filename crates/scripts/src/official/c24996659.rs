// Wattkiwi
use crate::prelude::*;
use crate::testing::{Check, MZONE};
use engine::constants::MSG_ATTACK;

const ID: u32 = 24996659;

pub const LISTED_SERIES: &[u16] = &[SET_WATT];
// Wattkid and Wattaildragon are Normal Monsters of the archetype.
pub const SUPPORT: &[(u32, u16)] = &[(ID, MZONE), (27324313, MZONE), (87151205, MZONE)];
pub const CHECKS: &[Check] = &[Check::Message(MSG_ATTACK)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_code(duel, EFFECT_INDESTRUCTABLE_BATTLE);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_target_range(duel, LOCATION_MZONE, 0);
    e1.set_target_card(duel, indtg);
    e1.set_value(duel, 1);
    c.register_effect(duel, e1);
}

fn indtg(duel: &Duel, _: EffectId, c: CardId) -> bool {
    c.is_set_card(duel, &[SET_WATT]) && duel.get_attacker() == Some(c)
}
