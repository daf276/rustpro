// Rikka Glamour
use crate::prelude::*;
use crate::testing::{Check, DECK, HAND, MZONE};

const ID: u32 = 69164989;

pub const LISTED_SERIES: &[u16] = &[SET_RIKKA];
// Level 4 Rikka Plants (Primula, Princess) and Snakeyashi in the Deck, and Plants on the field to Tribute.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (8129306, DECK),
    (8129306, DECK),
    (132308, DECK),
    (29802344, DECK),
    (29802344, MZONE),
    (29802344, MZONE),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_TOHAND + CATEGORY_SEARCH);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_count_limit_full(duel, 1, ID, EFFECT_COUNT_CODE_OATH);
    e1.set_cost(duel, cost);
    e1.set_target(duel, target);
    e1.set_operation(duel, operation);
    e1.set_label(duel, 0);
    c.register_effect(duel, e1);
}

fn filter(c: CardId, duel: &Duel, tp: u8, check: i64) -> bool {
    c.is_set_card(duel, &[SET_RIKKA])
        && c.is_monster(duel)
        && c.is_able_to_hand(duel)
        && (check == 0 || {
            let (lv, code) = (c.get_original_level(duel), c.get_code(duel));
            duel.is_existing_matching_card(|c, duel| filter2(c, duel, lv, code), tp, LOCATION_DECK, 0, 1, None)
        })
}

fn filter2(c: CardId, duel: &Duel, lv: u32, code: u32) -> bool {
    c.is_race(duel, RACE_PLANT)
        && c.is_monster(duel)
        && c.is_able_to_hand(duel)
        && c.get_original_level(duel) == lv
        && !c.is_code(duel, code)
}

fn exists(duel: &Duel, tp: u8, check: i64) -> bool {
    duel.is_existing_matching_card(|c, duel| filter(c, duel, tp, check), tp, LOCATION_DECK, 0, 1, None)
}

fn cost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let (a_exists, b_exists) = (exists(duel, tp, 0), exists(duel, tp, 1));
    if chk == 0 {
        return Ok(a_exists || b_exists);
    }
    let releasable = |c: CardId, duel: &Duel| c.is_rikka_releasable(duel, tp);
    if b_exists
        && duel.check_release_group_cost(tp, releasable, (1, 1), false, None, None)
        && duel.select_yes_no(tp, aux::stringid(ID, 1))?
    {
        let g = duel.select_release_group_cost(tp, releasable, (1, 1), false, None, None)?;
        duel.release(g, REASON_COST)?;
        e.set_label(duel, 1);
    } else {
        e.set_label(duel, 0);
    }
    Ok(false)
}

fn target(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(exists(duel, a.tp, 0));
    }
    let count = a.e.get_label(duel) + 1;
    duel.set_operation_info(0, CATEGORY_TOHAND, None, count as u8, a.tp, LOCATION_DECK as i32);
    Ok(false)
}

fn operation(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let label = e.get_label(duel);
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_ATOHAND);
    let g = duel.select_matching_card(tp, |c, duel| filter(c, duel, tp, label), tp, LOCATION_DECK, 0, 1, 1, None)?;
    let Some(first) = g.get_first() else { return Ok(()) };
    let (lv, code) = (first.get_original_level(duel), first.get_code(duel));
    duel.send_to_hand(g, PLAYER_NONE, REASON_EFFECT)?;
    duel.confirm_cards(1 - tp, g)?;
    if e.get_label(duel) == 1 {
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_ATOHAND);
        let sg = duel.select_matching_card(tp, |c, duel| filter2(c, duel, lv, code), tp, LOCATION_DECK, 0, 1, 1, None)?;
        if !sg.is_empty() {
            duel.break_effect()?;
            duel.send_to_hand(sg, PLAYER_NONE, REASON_EFFECT)?;
            duel.confirm_cards(1 - tp, sg)?;
        }
    }
    Ok(())
}
