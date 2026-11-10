// Blade Knight
use engine::constants::MSG_ATTACK;

use crate::prelude::*;
use crate::testing::{Check, MZONE, OPPONENT};

const ID: u32 = 39507162;

// Alone on the field against an opponent's monster (Elfnote Lucina) for battles.
pub const SUPPORT: &[(u32, u16)] = &[(ID, MZONE), (13597785, MZONE | OPPONENT)];
pub const CHECKS: &[Check] = &[Check::Message(MSG_ATTACK)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_property(duel, EFFECT_FLAG_SINGLE_RANGE);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_code(duel, EFFECT_UPDATE_ATTACK);
    e1.set_value(duel, 400);
    e1.set_condition(duel, |duel, a| {
        duel.get_field_group_count(a.e.get_handler_player(duel), LOCATION_HAND, 0) <= 1
    });
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_CONTINUOUS);
    e2.set_code(duel, EVENT_BATTLED);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_condition(duel, discon);
    e2.set_operation(duel, disop);
    c.register_effect(duel, e2);
}

fn discon(duel: &Duel, a: &Args) -> bool {
    let c = a.e.get_handler(duel);
    !duel.is_existing_matching_card(aux::TRUE, a.e.get_handler_player(duel), LOCATION_MZONE, 0, 1, Some(c.into()))
}

fn disop(duel: &mut Duel, a: &Args) -> Script<()> {
    let c = a.e.get_handler(duel);
    let Some(bc) = c.get_battle_target(duel) else { return Ok(()) };
    if bc.is_status(duel, STATUS_BATTLE_DESTROYED) && bc.is_type(duel, TYPE_FLIP) {
        for code in [EFFECT_DISABLE, EFFECT_DISABLE_EFFECT] {
            let e = duel.create_effect(c);
            e.set_type(duel, EFFECT_TYPE_SINGLE);
            e.set_code(duel, code);
            e.set_reset(duel, RESET_EVENT | RESETS_STANDARD_EXC_GRAVE);
            bc.register_effect(duel, e);
        }
    }
    Ok(())
}
