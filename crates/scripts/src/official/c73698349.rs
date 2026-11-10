// Giant Orc
use engine::constants::MSG_POS_CHANGE;

use crate::prelude::*;
use crate::testing::{Check, MZONE, OPPONENT};

const ID: u32 = 73698349;

// On the field attacking Sangan of the opponent's.
pub const SUPPORT: &[(u32, u16)] = &[(ID, MZONE), (26202165, MZONE | OPPONENT)];
pub const CHECKS: &[Check] = &[Check::Message(MSG_POS_CHANGE)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_CONTINUOUS);
    e1.set_code(duel, EVENT_PHASE | PHASE_BATTLE);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_count_limit(duel, 1, 0);
    e1.set_condition(duel, |duel, a| a.e.get_handler(duel).get_attacked_count(duel) > 0);
    e1.set_operation(duel, posop);
    c.register_effect(duel, e1);
}

fn posop(duel: &mut Duel, a: &Args) -> Script<()> {
    let c = a.e.get_handler(duel);
    if c.is_attack_pos(duel) {
        duel.change_position(c, POS_FACEUP_DEFENSE)?;
    }
    let e1 = duel.create_effect(c);
    e1.set_description(duel, 3313);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_code(duel, EFFECT_CANNOT_CHANGE_POSITION);
    e1.set_property(
        duel,
        EFFECT_FLAG_CANNOT_DISABLE + EFFECT_FLAG_COPY_INHERIT + EFFECT_FLAG_CLIENT_HINT,
    );
    e1.set_reset_full(duel, RESETS_STANDARD_PHASE_END, 3);
    c.register_effect(duel, e1);
    Ok(())
}
