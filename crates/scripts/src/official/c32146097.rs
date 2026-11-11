// D/D Pandora
use crate::prelude::*;
use crate::testing::{Check, HAND, MZONE, OPPONENT};

const ID: u32 = 32146097;

// Alone on the field, destroyed by the opponent's Dark Hole.
pub const SUPPORT: &[(u32, u16)] = &[(ID, MZONE), (53129443, HAND | OPPONENT)];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(0)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_category(duel, CATEGORY_DRAW);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e1.set_code(duel, EVENT_TO_GRAVE);
    e1.set_property(duel, EFFECT_FLAG_PLAYER_TARGET);
    e1.set_condition(duel, drcon);
    e1.set_target(duel, stock::draw_target::<2>);
    e1.set_operation(duel, stock::draw);
    c.register_effect(duel, e1);
}

fn drcon(duel: &Duel, a: &Args) -> bool {
    let (c, tp) = (a.e.get_handler(duel), a.tp);
    (c.is_reason(duel, REASON_BATTLE) || a.rp != tp && c.is_reason(duel, REASON_DESTROY) && c.is_previous_controler(duel, tp))
        && duel.get_field_group_count(tp, LOCATION_ONFIELD, 0) == 0
}
