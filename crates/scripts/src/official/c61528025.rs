// Banisher of the Light
use engine::constants::MSG_ATTACK;

use crate::prelude::*;
use crate::testing::{Check, HAND, MZONE, OPPONENT};

const ID: u32 = 61528025;

// On the field against the opponent's Raigeki and Dark Hole, which send monsters to the Graveyard.
pub const SUPPORT: &[(u32, u16)] = &[(ID, MZONE), (12580477, HAND | OPPONENT), (53129443, HAND | OPPONENT)];
// A continuous effect only, so there is nothing to resolve.
pub const CHECKS: &[Check] = &[Check::Message(MSG_ATTACK)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_property(
        duel,
        EFFECT_FLAG_SET_AVAILABLE + EFFECT_FLAG_IGNORE_RANGE + EFFECT_FLAG_IGNORE_IMMUNE,
    );
    e1.set_code(duel, EFFECT_TO_GRAVE_REDIRECT);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_target_range(duel, 0xff, 0xff);
    e1.set_target_card(duel, |duel, e, c| duel.is_player_can_remove(e.get_handler_player(duel), Some(c)));
    e1.set_value(duel, LOCATION_REMOVED as i32);
    c.register_effect(duel, e1);
}
