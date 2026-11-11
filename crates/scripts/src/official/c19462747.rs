// Blackwing - Decay the Ill Wind
use crate::prelude::*;
use crate::testing::{Check, HAND, MZONE};

const ID: u32 = 19462747;

// In hand and on the field with Gale, whose Level it changes, and Dark Hole in hand to send it to the Graveyard.
pub const SUPPORT: &[(u32, u16)] = &[(ID, HAND), (ID, MZONE), (2009101, MZONE), (53129443, HAND)];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 3)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e1.set_code(duel, EVENT_SUMMON_SUCCESS);
    e1.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e1.set_count_limit(duel, 1, ID);
    e1.set_target(duel, target);
    e1.set_operation(duel, operation);
    c.register_effect(duel, e1);
    let e2 = e1.clone_effect(duel);
    e2.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_CONTINUOUS);
    e3.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE);
    e3.set_code(duel, EVENT_TO_GRAVE);
    e3.set_operation(duel, regop);
    c.register_effect(duel, e3);
    let e4 = duel.create_effect(c);
    e4.set_description(duel, aux::stringid(ID, 3));
    e4.set_category(duel, CATEGORY_DAMAGE);
    e4.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_F);
    e4.set_code(duel, EVENT_PHASE + PHASE_END);
    e4.set_count_limit(duel, 1, 0);
    e4.set_range(duel, LOCATION_GRAVE);
    e4.set_condition(duel, damcon);
    e4.set_target(duel, damtg);
    e4.set_operation(duel, stock::damage);
    c.register_effect(duel, e4);
}

fn filter(c: CardId, duel: &Duel) -> bool {
    c.is_faceup(duel) && c.has_level(duel) && c.is_set_card(duel, &[SET_BLACKWING])
}

fn target(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_MZONE) && chkc.is_controler(duel, tp) && filter(chkc, duel));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(filter, tp, LOCATION_MZONE, 0, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_FACEUP);
    let g = duel.select_target(tp, filter, tp, LOCATION_MZONE, 0, 1, 1, None)?;
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_EFFECT);
    let op = if g.and_then(|g| g.get_first()).is_some_and(|tc| tc.get_level(duel) == 1) {
        duel.select_option(tp, &[aux::stringid(ID, 1)])?
    } else {
        duel.select_option(tp, &[aux::stringid(ID, 1), aux::stringid(ID, 2)])?
    };
    e.set_label(duel, i64::from(op));
    Ok(false)
}

fn operation(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, c) = (a.e, a.e.get_handler(duel));
    if let Some(tc) = duel.get_first_target()
        && tc.is_faceup(duel)
        && tc.is_relate_to_effect(duel, e)
    {
        let e1 = duel.create_effect(c);
        e1.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE);
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_code(duel, EFFECT_UPDATE_LEVEL);
        e1.set_reset(duel, RESET_EVENT | RESETS_STANDARD);
        e1.set_value(duel, if e.get_label(duel) == 0 { 1 } else { -1 });
        tc.register_effect(duel, e1);
    }
    Ok(())
}

fn regop(duel: &mut Duel, a: &Args) -> Script<()> {
    a.e.get_handler(duel)
        .register_flag_effect(duel, ID, RESETS_STANDARD_PHASE_END, 0, 1);
    Ok(())
}

fn damcon(duel: &Duel, a: &Args) -> bool {
    a.e.get_handler(duel).get_flag_effect(duel, ID) > 0
}

fn damtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(true);
    }
    duel.set_target_player(tp);
    duel.set_target_param(1000);
    duel.set_operation_info(0, CATEGORY_DAMAGE, None, 0, tp, 1000);
    Ok(false)
}
