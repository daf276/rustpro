// Fabled Raven
use crate::prelude::*;
use crate::testing::{HAND, MZONE};

const ID: u32 = 47217354;

// On the field with Fabled Lurrie in the hand to discard.
pub const SUPPORT: &[(u32, u16)] = &[(ID, MZONE), (97651498, HAND), (97651498, HAND)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_HANDES + CATEGORY_ATKCHANGE);
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_count_limit(duel, 1, 0);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_target(duel, tg);
    e1.set_operation(duel, op);
    c.register_effect(duel, e1);
}

fn tg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.get_field_group_count(tp, LOCATION_HAND, 0) > 0);
    }
    duel.set_operation_info(0, CATEGORY_HANDES, None, 0, tp, 1);
    Ok(false)
}

fn op(duel: &mut Duel, a: &Args) -> Script<()> {
    let (c, tp) = (a.e.get_handler(duel), a.tp);
    let ct = duel.discard_hand(tp, aux::TRUE, 1, 60, REASON_EFFECT | REASON_DISCARD)?;
    if ct > 0 && c.is_faceup(duel) && c.is_relate_to_effect(duel, a.e) {
        let e1 = duel.create_effect(c);
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_code(duel, EFFECT_UPDATE_ATTACK);
        e1.set_reset(duel, RESETS_STANDARD_DISABLE_PHASE_END);
        e1.set_value(duel, ct * 400);
        c.register_effect(duel, e1);
        let e2 = duel.create_effect(c);
        e2.set_type(duel, EFFECT_TYPE_SINGLE);
        e2.set_code(duel, EFFECT_UPDATE_LEVEL);
        e2.set_reset(duel, RESETS_STANDARD_DISABLE_PHASE_END);
        e2.set_value(duel, ct);
        c.register_effect(duel, e2);
    }
    Ok(())
}
