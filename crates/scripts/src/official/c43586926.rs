// Twin-Headed Behemoth
use crate::prelude::*;
use crate::testing::{HAND, MZONE, OPPONENT};

const ID: u32 = 43586926;

// On the field, destroyed by the opponent's Raigeki and Dark Hole.
pub const SUPPORT: &[(u32, u16)] = &[(ID, MZONE), (ID, MZONE), (12580477, HAND | OPPONENT), (53129443, HAND | OPPONENT)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_CONTINUOUS);
    e1.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE);
    e1.set_code(duel, EVENT_TO_GRAVE);
    e1.set_operation(duel, regop);
    c.register_effect(duel, e1);
}

fn regop(duel: &mut Duel, a: &Args) -> Script<()> {
    let c = a.e.get_handler(duel);
    if c.is_reason(duel, REASON_DESTROY) && c.get_previous_location(duel) & LOCATION_ONFIELD != 0 {
        let e1 = duel.create_effect(c);
        e1.set_description(duel, aux::stringid(ID, 0));
        e1.set_category(duel, CATEGORY_SPECIAL_SUMMON);
        e1.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
        e1.set_range(duel, LOCATION_GRAVE);
        e1.set_code(duel, EVENT_PHASE + PHASE_END);
        e1.set_count_limit_full(duel, 1, ID, EFFECT_COUNT_CODE_DUEL);
        e1.set_target(duel, sptg);
        e1.set_operation(duel, spop);
        e1.set_reset(duel, RESETS_STANDARD_PHASE_END);
        c.register_effect(duel, e1);
    }
    Ok(())
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(true);
    }
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, Some(a.e.get_handler(duel).into()), 1, 0, 0);
    Ok(false)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if c.is_relate_to_effect(duel, e) && duel.special_summon(c, 0, tp, tp, false, false, POS_FACEUP)? != 0 {
        let e1 = duel.create_effect(c);
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_code(duel, EFFECT_SET_ATTACK);
        e1.set_value(duel, 1000);
        e1.set_reset(duel, RESET_EVENT | RESETS_STANDARD_DISABLE);
        c.register_effect(duel, e1);
        let e2 = e1.clone_effect(duel);
        e2.set_code(duel, EFFECT_SET_DEFENSE);
        c.register_effect(duel, e2);
    }
    Ok(())
}
