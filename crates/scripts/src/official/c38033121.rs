// Dark Magician Girl
use engine::constants::MSG_ATTACK;

use crate::prelude::*;
use crate::testing::{Check, GRAVE, MZONE, OPPONENT};

const ID: u32 = 38033121;
const MAGICIAN_OF_BLACK_CHAOS: u32 = 30208479;

pub const LISTED_NAMES: &[u32] = &[CARD_DARK_MAGICIAN];

// On the field and attacking Sangan of the opponent's, with Dark Magician and Magician of Black Chaos in the Graveyard to boost it.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (CARD_DARK_MAGICIAN, GRAVE),
    (MAGICIAN_OF_BLACK_CHAOS, GRAVE),
    (26202165, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[Check::Message(MSG_ATTACK)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_property(duel, EFFECT_FLAG_SINGLE_RANGE);
    e1.set_code(duel, EFFECT_UPDATE_ATTACK);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_value(duel, Value::Card(val));
    c.register_effect(duel, e1);
}

fn val(duel: &Duel, _: EffectId, c: CardId) -> i32 {
    let filter = |c: CardId, duel: &Duel| c.is_code(duel, MAGICIAN_OF_BLACK_CHAOS) || c.is_code(duel, CARD_DARK_MAGICIAN);
    (duel.get_matching_group_count(filter, c.get_controler(duel), LOCATION_GRAVE, LOCATION_GRAVE, None) * 300) as i32
}
