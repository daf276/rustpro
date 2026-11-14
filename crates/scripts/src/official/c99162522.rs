// EXTINGUISH!
use crate::prelude::*;
use crate::testing::{Check, HAND, MZONE, OPPONENT, SZONE};

const ID: u32 = 99162522;

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
    e1.set_category(duel, CATEGORY_DESTROY);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_hint_timing(duel, 0, TIMINGS_CHECK_MONSTER_E);
    e1.set_condition(duel, condition);
    e1.set_target(duel, target);
    e1.set_operation(duel, activate);
    c.register_effect(duel, e1);
}

pub fn condition(duel: &Duel, a: &Args) -> bool {
    duel.is_existing_matching_card(
        aux::faceup_filter(|c, duel| c.is_set_card(duel, &[SET_RESCUE_ACE])),
        a.tp,
        LOCATION_MZONE,
        0,
        1,
        None,
    )
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
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_DESTROY);
    let g = duel.select_target(tp, filter, tp, 0, LOCATION_MZONE, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_DESTROY, g, 1, tp, 0);
    Ok(false)
}

pub fn hydrant_check(duel: &Duel, tp: u8) -> bool {
    duel.is_existing_matching_card(
        aux::faceup_filter(|c, duel| c.is_code(duel, CARD_R_ACE_HYDRANT)),
        tp,
        LOCATION_ONFIELD,
        0,
        1,
        None,
    )
}

fn activate(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let Some(tc) = duel.get_first_target() else { return Ok(()) };
    if !tc.is_relate_to_effect(duel, e) || duel.destroy(tc, REASON_EFFECT)? == 0 {
        return Ok(());
    }
    if !hydrant_check(duel, tp) {
        return Ok(());
    }
    tc.register_flag_effect(duel, ID, RESETS_STANDARD_PHASE_END, 0, 1);
    let code = tc.get_original_code(duel);
    let e1 = duel.create_effect(e.get_handler(duel));
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_code(duel, EFFECT_CANNOT_ACTIVATE);
    e1.set_property(duel, EFFECT_FLAG_PLAYER_TARGET);
    e1.set_target_range(duel, 0, 1);
    e1.set_value(duel, Value::EffectPlayer(aclimit));
    e1.set_label(duel, i64::from(code));
    e1.set_reset(duel, RESET_PHASE | PHASE_END);
    duel.register_effect(e1, tp);
    Ok(())
}

fn aclimit(duel: &Duel, e: EffectId, re: EffectId, _: u8) -> bool {
    let rc = re.get_handler(duel);
    re.is_monster_effect(duel) && (rc.get_flag_effect(duel, ID) > 0 || i64::from(rc.get_original_code(duel)) == e.get_label(duel))
}
