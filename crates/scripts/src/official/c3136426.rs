// Level Limit - Area B
use crate::prelude::*;
use crate::testing::{HAND, MZONE, OPPONENT};

const ID: u32 = 3136426;

// In the hand, with monsters above Level 3 on both sides: Elfnote Lucina and Dark Magician.
pub const SUPPORT: &[(u32, u16)] = &[(ID, HAND), (13597785, MZONE), (CARD_DARK_MAGICIAN, MZONE | OPPONENT)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_FIELD);
    e2.set_code(duel, EFFECT_SET_POSITION);
    e2.set_range(duel, LOCATION_SZONE);
    e2.set_target_card(duel, |duel, _, c| c.get_level(duel) > 3 && c.is_faceup(duel));
    e2.set_target_range(duel, LOCATION_MZONE, LOCATION_MZONE);
    e2.set_value(duel, POS_FACEUP_DEFENSE as i32);
    c.register_effect(duel, e2);
}
