// Dewdark of the Ice Barrier
use engine::constants::MSG_ATTACK;

use crate::prelude::*;
use crate::testing::{Check, MZONE, OPPONENT};

const ID: u32 = 90311614;

// On the field, with a monster on the opponent's side to attack past.
pub const SUPPORT: &[(u32, u16)] = &[(ID, MZONE), (ID, MZONE), (26202165, MZONE | OPPONENT)];
// It has no effect to resolve.
pub const CHECKS: &[Check] = &[Check::Message(MSG_ATTACK)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_code(duel, EFFECT_DIRECT_ATTACK);
    e1.set_condition(duel, dircon);
    c.register_effect(duel, e1);
}

fn filter(c: CardId, duel: &Duel) -> bool {
    let lv = c.get_level(duel);
    c.is_faceup(duel) && (lv == 0 || lv > 2)
}

fn dircon(duel: &Duel, a: &Args) -> bool {
    let tp = a.e.get_handler(duel).get_controler(duel);
    !duel.is_existing_matching_card(filter, tp, LOCATION_MZONE, 0, 1, None)
}
