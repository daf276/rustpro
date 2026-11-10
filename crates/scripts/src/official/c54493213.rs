// Helios - The Primordial Sun
use engine::constants::MSG_ATTACK;

use crate::prelude::*;
use crate::testing::{Check, DECK, HAND, MZONE};

const ID: u32 = 54493213;

// Macro Cosmos banishes cards, and Helios on the field attacks with its ATK from them.
pub const SUPPORT: &[(u32, u16)] = &[(ID, MZONE), (ID, DECK), (30241314, HAND), (30241314, HAND)];
pub const CHECKS: &[Check] = &[Check::Message(MSG_ATTACK)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_property(duel, EFFECT_FLAG_SINGLE_RANGE);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_code(duel, EFFECT_SET_ATTACK);
    e1.set_value(duel, Value::Card(value));
    c.register_effect(duel, e1);
    let e2 = e1.clone_effect(duel);
    e2.set_code(duel, EFFECT_SET_DEFENSE);
    c.register_effect(duel, e2);
}

fn filter(c: CardId, duel: &Duel) -> bool {
    c.is_faceup(duel) && c.is_monster(duel)
}

fn value(duel: &Duel, _: EffectId, c: CardId) -> i32 {
    duel.get_matching_group_count(filter, c.get_controler(duel), LOCATION_REMOVED, LOCATION_REMOVED, None) as i32 * 100
}
