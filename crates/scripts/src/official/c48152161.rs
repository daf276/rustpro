// The Sanctified Darklord
use crate::prelude::*;
use crate::testing::{Check, HAND, MZONE, SZONE};

const ID: u32 = 48152161;

pub const LISTED_SERIES: &[u16] = &[SET_DARKLORD];

const DARKLORD_GULGOLET: u32 = 84031359;
const DARKLORD_DJEHUTY: u32 = 10426067;

// Set, with Darklords in the hand to send and an Effect Monster on the field to negate.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, SZONE),
    (DARKLORD_GULGOLET, HAND),
    (DARKLORD_DJEHUTY, HAND),
    (DARKLORD_DJEHUTY, MZONE),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_DISABLE + CATEGORY_RECOVER);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_count_limit_full(duel, 1, ID, EFFECT_COUNT_CODE_OATH);
    e1.set_hint_timing(duel, 0, TIMING_STANDBY_PHASE | TIMING_MAIN_END | TIMINGS_CHECK_MONSTER_E);
    e1.set_cost(duel, cost);
    e1.set_target(duel, target);
    e1.set_operation(duel, activate);
    c.register_effect(duel, e1);
}

fn costfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_DARKLORD])
        && c.is_monster(duel)
        && (c.is_location(duel, LOCATION_HAND) || c.is_faceup(duel))
        && c.is_able_to_grave_as_cost(duel)
        && duel.is_existing_matching_card(disfilter, 0, LOCATION_MZONE, LOCATION_MZONE, 1, Some(c.into()))
}

fn disfilter(c: CardId, duel: &Duel) -> bool {
    c.is_negatable_monster(duel) && c.is_type(duel, TYPE_EFFECT)
}

fn cost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    stock::send_to_grave_cost(duel, a, chk, costfilter, LOCATION_HAND | LOCATION_MZONE)
}

fn target(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(disfilter, tp, LOCATION_MZONE, LOCATION_MZONE, 1, None));
    }
    duel.set_operation_info(0, CATEGORY_DISABLE, None, 1, tp, 0);
    Ok(false)
}

fn activate(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_NEGATE);
    let Some(sc) = duel
        .select_matching_card(tp, disfilter, tp, LOCATION_MZONE, LOCATION_MZONE, 1, 1, None)?
        .get_first()
    else {
        return Ok(());
    };
    duel.hint_selection(sc);
    if sc.is_can_be_disabled_by_effect(duel, e, true) {
        sc.negate_effects(duel, e.get_handler(duel), RESETS_STANDARD_PHASE_END, false);
        duel.adjust_instantly(Some(sc));
        let atk = sc.get_attack(duel);
        if atk > 0 {
            duel.recover(tp, atk as u32, REASON_EFFECT)?;
        }
    }
    Ok(())
}
