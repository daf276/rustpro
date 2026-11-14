// CONTAIN!
use crate::prelude::*;
use crate::testing::{Check, HAND, MZONE, OPPONENT, SZONE};

use super::c99162522::{condition, hydrant_check};

const ID: u32 = 62777823;

pub const LISTED_SERIES: &[u16] = &[SET_RESCUE_ACE];
pub const LISTED_NAMES: &[u32] = &[CARD_R_ACE_HYDRANT];

const RESCUE_ACE_IMPULSE: u32 = 38339996;

// Set, with Rescue-ACE Impulse and Hydrant on the field, against the opponent's Ash Blossom and Elfnote Lucina.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, SZONE),
    (ID, SZONE),
    (ID, HAND),
    (RESCUE_ACE_IMPULSE, MZONE),
    (RESCUE_ACE_IMPULSE, MZONE),
    (CARD_R_ACE_HYDRANT, MZONE),
    (14558127, MZONE | OPPONENT),
    (13597785, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_DISABLE);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_hint_timing(duel, 0, TIMINGS_CHECK_MONSTER);
    e1.set_condition(duel, condition);
    e1.set_target(duel, target);
    e1.set_operation(duel, activate);
    c.register_effect(duel, e1);
}

fn target(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    let filter = aux::faceup_filter(|c, duel| c.is_type(duel, TYPE_EFFECT));
    if let Some(chkc) = chkc {
        return Ok(chkc.is_controler(duel, 1 - tp) && chkc.is_location(duel, LOCATION_MZONE) && filter(chkc, duel));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(filter, tp, 0, LOCATION_MZONE, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_FACEUP);
    duel.select_target(tp, filter, tp, 0, LOCATION_MZONE, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_DISABLE, a.eg, 1, 0, 0);
    Ok(false)
}

fn activate(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let Some(tc) = duel.get_first_target() else { return Ok(()) };
    if !(tc.is_relate_to_effect(duel, e) && tc.is_faceup(duel) && tc.is_type(duel, TYPE_EFFECT)) {
        return Ok(());
    }
    duel.negate_related_chain(tc, RESET_TURN_SET);
    let c = e.get_handler(duel);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_code(duel, EFFECT_DISABLE);
    e1.set_reset(duel, RESETS_STANDARD_PHASE_END);
    tc.register_effect(duel, e1);
    let e2 = e1.clone_effect(duel);
    e2.set_code(duel, EFFECT_DISABLE_EFFECT);
    e2.set_value(duel, RESET_TURN_SET as i32);
    tc.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, 3206);
    e3.set_type(duel, EFFECT_TYPE_SINGLE);
    e3.set_property(duel, EFFECT_FLAG_CLIENT_HINT);
    e3.set_code(duel, EFFECT_CANNOT_ATTACK);
    e3.set_value(duel, 1);
    e3.set_reset(duel, RESETS_STANDARD_PHASE_END);
    tc.register_effect(duel, e3);
    if !hydrant_check(duel, tp) {
        return Ok(());
    }
    let e4 = duel.create_effect(c);
    e4.set_description(duel, aux::stringid(ID, 1));
    e4.set_type(duel, EFFECT_TYPE_SINGLE);
    e4.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE + EFFECT_FLAG_UNCOPYABLE + EFFECT_FLAG_CLIENT_HINT);
    e4.set_code(duel, EFFECT_CANNOT_BE_MATERIAL);
    e4.set_value(
        duel,
        Value::Material(|_, _, _, sumtype, _| {
            let allowed = [SUMMON_TYPE_FUSION, SUMMON_TYPE_SYNCHRO, SUMMON_TYPE_XYZ, SUMMON_TYPE_LINK];
            aux::cannotmatfilter(&allowed.map(u64::from), sumtype)
        }),
    );
    e4.set_reset(duel, RESETS_STANDARD_PHASE_END);
    tc.register_effect(duel, e4);
    Ok(())
}
