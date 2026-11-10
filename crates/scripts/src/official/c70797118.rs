// Thunder Nyan Nyan
use engine::constants::MSG_MOVE;

use crate::prelude::*;
use crate::testing::{Check, MZONE};

const ID: u32 = 70797118;

// With non-LIGHT monsters (Doron) of yours on the field to make it self-destroy (a move message, as destruction has none).
pub const SUPPORT: &[(u32, u16)] = &[(ID, 0), (756652, MZONE), (756652, MZONE), (756652, MZONE)];
pub const CHECKS: &[Check] = &[Check::Message(MSG_MOVE)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_property(duel, EFFECT_FLAG_SINGLE_RANGE);
    e1.set_code(duel, EFFECT_SELF_DESTROY);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_condition(duel, |duel, a| {
        let filter = aux::faceup_filter(|c, duel| c.is_attribute_except(duel, ATTRIBUTE_LIGHT));
        duel.is_existing_matching_card(filter, a.e.get_handler_player(duel), LOCATION_MZONE, 0, 1, None)
    });
    c.register_effect(duel, e1);
}
