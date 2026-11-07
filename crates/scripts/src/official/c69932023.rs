// Dracotail Horn
use super::c5431722::{place_and_draw, place_and_draw_info};
use crate::prelude::*;
use crate::testing::{Check, DECK, GRAVE, HAND, MZONE, OPPONENT};

const ID: u32 = 69932023;

pub const LISTED_NAMES: &[u32] = &[ID];

// Attack Position monsters of both players, one of them from the Extra Deck, and a Dracotail monster in the Graveyard to place
// on the bottom of the Deck.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (13597785, MZONE | OPPONENT),
    (33760966, MZONE),
    (75003700, GRAVE),
    (75003700, DECK),
];
pub const CHECKS: &[Check] = &[Check::Resolves];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_TOHAND + CATEGORY_TOEXTRA + CATEGORY_TODECK + CATEGORY_DRAW);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_hint_timing(duel, 0, TIMING_STANDBY_PHASE | TIMING_MAIN_END | TIMINGS_CHECK_MONSTER_E);
    e1.set_count_limit_full(duel, 1, ID, EFFECT_COUNT_CODE_OATH);
    e1.set_target(duel, target);
    e1.set_operation(duel, activate);
    c.register_effect(duel, e1);
}

fn thexfilter(c: CardId, duel: &Duel) -> bool {
    c.is_attack_pos(duel) && (c.is_able_to_hand(duel) || c.is_able_to_extra(duel))
}

fn target(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_MZONE) && thexfilter(chkc, duel));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(thexfilter, tp, LOCATION_MZONE, LOCATION_MZONE, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_RTOHAND);
    let Some(tc) = duel
        .select_target(tp, thexfilter, tp, LOCATION_MZONE, LOCATION_MZONE, 1, 1, None)?
        .and_then(|g| g.get_first())
    else {
        return Ok(false);
    };
    let category = if tc.is_type(duel, TYPE_EXTRA) {
        CATEGORY_TOEXTRA
    } else {
        CATEGORY_TOHAND
    };
    duel.set_operation_info(0, category, Some(tc.into()), 1, tp, 0);
    place_and_draw_info(duel, tp);
    Ok(false)
}

fn activate(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let Some(tc) = duel.get_first_target() else {
        return Ok(());
    };
    if tc.is_relate_to_effect(duel, e)
        && duel.send_to_hand(tc, PLAYER_NONE, REASON_EFFECT)? > 0
        && tc.is_location(duel, LOCATION_HAND | LOCATION_EXTRA)
    {
        place_and_draw::<ID>(duel, tp)?;
    }
    Ok(())
}
