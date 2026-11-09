// Atlantean Attack Squad
use engine::constants::MSG_ATTACK;

use crate::prelude::*;
use crate::testing::{Check, MZONE};

const ID: u32 = 8078366;

// Two copies on the field with Atlantean Pikeman, a Sea Serpent Normal Monster, to power each other up.
pub const SUPPORT: &[(u32, u16)] = &[(ID, MZONE), (ID, MZONE), (26976414, MZONE)];
// It has no effect to resolve.
pub const CHECKS: &[Check] = &[Check::Message(MSG_ATTACK)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_code(duel, EFFECT_UPDATE_ATTACK);
    e1.set_property(duel, EFFECT_FLAG_SINGLE_RANGE);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_condition(duel, atkcon);
    e1.set_value(duel, 800);
    c.register_effect(duel, e1);
}

fn atkcon(duel: &Duel, a: &Args) -> bool {
    let c = a.e.get_handler(duel);
    duel.is_existing_matching_card(
        aux::faceup_filter(|c, duel| c.is_race(duel, RACE_FISH | RACE_SEASERPENT | RACE_AQUA)),
        a.e.get_handler_player(duel),
        LOCATION_MZONE,
        0,
        1,
        Some(c.into()),
    )
}
