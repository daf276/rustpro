// Swallow's Cowrie
use crate::prelude::*;
use crate::testing::{Check, DECK, HAND};

const ID: u32 = 5993144;

// Level 2 Winged Beasts (Harpie Girl, Droll Bird) in the hand to release and in the Deck to add.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (34100324, HAND),
    (97973387, HAND),
    (34100324, DECK),
    (97973387, DECK),
    (97973387, DECK),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_SEARCH + CATEGORY_TOHAND);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_count_limit_full(duel, 1, ID, EFFECT_COUNT_CODE_OATH);
    e1.set_hint_timing(duel, 0, TIMING_MAIN_END | TIMINGS_CHECK_MONSTER_E);
    e1.set_cost(duel, cost);
    e1.set_target(duel, target);
    e1.set_operation(duel, activate);
    c.register_effect(duel, e1);
}

fn costfilter(c: CardId, duel: &Duel, tp: u8) -> bool {
    c.is_race(duel, RACE_WINGEDBEAST)
        && c.has_level(duel)
        && (c.is_faceup(duel) || c.is_location(duel, LOCATION_HAND))
        && duel.is_existing_matching_card(|tc, duel| thfilter(tc, duel, c.get_level(duel)), tp, LOCATION_DECK, 0, 1, None)
}

fn thfilter(c: CardId, duel: &Duel, lv: u32) -> bool {
    c.is_race(duel, RACE_WINGEDBEAST) && c.is_level(duel, &[lv]) && c.is_able_to_hand(duel)
}

fn cost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let tp = a.tp;
    a.e.set_label(duel, 100);
    let f = |c, duel: &Duel| costfilter(c, duel, tp);
    if chk == 0 {
        return Ok(duel.check_release_group_cost(tp, f, (1, 1), true, None, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_RELEASE);
    let Some(tc) = duel.select_release_group_cost(tp, f, (1, 1), true, None, None)?.get_first() else {
        unported!(duel, true);
        return Ok(false);
    };
    let level = tc.get_level(duel);
    a.e.set_label(duel, i64::from(level));
    duel.release(tc, REASON_COST)?;
    Ok(false)
}

fn target(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(a.e.get_label(duel) == 100);
    }
    duel.set_operation_info(0, CATEGORY_TOHAND, None, 1, a.tp, LOCATION_DECK as i32);
    Ok(false)
}

fn activate(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    let lv = a.e.get_label(duel) as u32;
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_ATOHAND);
    let g = duel.select_matching_card(tp, |c, duel| thfilter(c, duel, lv), tp, LOCATION_DECK, 0, 1, 1, None)?;
    if !g.is_empty() {
        duel.send_to_hand(g, PLAYER_NONE, REASON_EFFECT)?;
        duel.confirm_cards(1 - tp, g)?;
    }
    Ok(())
}
