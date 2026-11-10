// Mudora
use engine::constants::MSG_ATTACK;

use crate::prelude::*;
use crate::testing::{Check, GRAVE, MZONE, OPPONENT};

const ID: u32 = 82108372;

// On the field to attack the opponent's monsters, with Fairy Normal Monsters in the Graveyard.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (1929294, GRAVE),
    (27288416, GRAVE),
    (14851496, MZONE | OPPONENT),
    (70781052, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[Check::Message(MSG_ATTACK)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_property(duel, EFFECT_FLAG_SINGLE_RANGE);
    e1.set_code(duel, EFFECT_UPDATE_ATTACK);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_value(
        duel,
        Value::Card(|duel, _, c| {
            200 * duel.get_matching_group_count(
                |c, duel| c.is_race(duel, RACE_FAIRY),
                c.get_controler(duel),
                LOCATION_GRAVE,
                0,
                None,
            ) as i32
        }),
    );
    c.register_effect(duel, e1);
}
