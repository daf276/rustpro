// Marincess Sea Star
use crate::prelude::*;
use crate::testing::{Check, HAND, MZONE};

const ID: u32 = 62886670;

pub const LISTED_SERIES: &[u16] = &[SET_MARINCESS];
// In hand with a Marincess Sea Star on the field to target.
pub const SUPPORT: &[(u32, u16)] = &[(ID, HAND), (ID, HAND), (ID, MZONE), (ID, MZONE)];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_ATKCHANGE);
    e1.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_range(duel, LOCATION_HAND);
    e1.set_count_limit(duel, 2, ID);
    e1.set_cost(duel, Cost::self_to_grave);
    e1.set_target(duel, target);
    e1.set_operation(duel, operation);
    c.register_effect(duel, e1);
}

fn filter(c: CardId, duel: &Duel) -> bool {
    c.is_faceup(duel) && c.is_set_card(duel, &[SET_MARINCESS])
}

fn target(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_controler(duel, tp) && chkc.is_location(duel, LOCATION_MZONE) && filter(chkc, duel));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(filter, tp, LOCATION_MZONE, 0, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_FACEUP);
    let g = duel.select_target(tp, filter, tp, LOCATION_MZONE, 0, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_ATKCHANGE, g, 1, tp, 800);
    Ok(false)
}

fn operation(duel: &mut Duel, a: &Args) -> Script<()> {
    if let Some(tc) = duel
        .get_first_target()
        .filter(|tc| tc.is_faceup(duel) && tc.is_relate_to_effect(duel, a.e))
    {
        let e1 = duel.create_effect(a.e.get_handler(duel));
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_code(duel, EFFECT_UPDATE_ATTACK);
        e1.set_reset(duel, RESETS_STANDARD_PHASE_END);
        e1.set_value(duel, 800);
        tc.register_effect(duel, e1);
    }
    Ok(())
}
