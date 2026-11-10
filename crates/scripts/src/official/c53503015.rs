// Frozen Rose
use crate::prelude::*;
use crate::testing::{Check, DECK, MZONE};

const ID: u32 = 53503015;

// The Lua's `s.check`, which the cost sets and the target resets.
const CHECK: u64 = (ID as u64) << 16;

// In the hand with a Plant (Angel Trumpeter) and a non-Plant (Mystical Elf) on the field to send, and Plant monsters of Level 4 or
// lower in the Deck to add.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, 0),
    (ID, 0),
    (87979586, MZONE),
    (15025844, MZONE),
    (22910685, DECK),
    (22910685, DECK),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_count_limit_full(duel, 1, ID, EFFECT_COUNT_CODE_OATH);
    e1.set_cost(duel, cost);
    e1.set_target(duel, target);
    e1.set_operation(duel, operation);
    c.register_effect(duel, e1);
}

fn cost(duel: &mut Duel, _: &Args, chk: i32) -> Script<bool> {
    duel.set_script_state(CHECK, 1);
    Ok(chk == 0)
}

fn cfilter(c: CardId, duel: &Duel, chk: u8, p: Option<bool>, chk1: bool, chk2: bool) -> bool {
    let plant = c.is_race(duel, RACE_PLANT);
    c.is_faceup(duel) && c.is_able_to_grave_as_cost(duel) && ((chk == 0 && Some(plant) == p) || ((plant && chk1) || (!plant && chk2)))
}

fn thfilter(c: CardId, duel: &Duel) -> bool {
    c.is_race(duel, RACE_PLANT) && c.is_level_below(duel, 4) && c.is_able_to_hand(duel)
}

fn target(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let chk1 = duel.is_existing_matching_card(
        |c, duel| cfilter(c, duel, 0, Some(true), false, false),
        tp,
        LOCATION_MZONE,
        0,
        1,
        None,
    );
    let chk2 = duel.is_existing_matching_card(
        |c, duel| cfilter(c, duel, 0, Some(false), false, false),
        tp,
        LOCATION_MZONE,
        0,
        1,
        None,
    ) && duel.is_existing_matching_card(thfilter, tp, LOCATION_DECK, 0, 1, None);
    if chk == 0 {
        if duel.get_script_state(CHECK) == 0 {
            return Ok(false);
        }
        duel.set_script_state(CHECK, 0);
        return Ok(chk1 || chk2);
    }
    e.set_label(duel, 0);
    e.set_category(duel, 0);
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TOGRAVE);
    let g = duel.select_matching_card(
        tp,
        |c, duel| cfilter(c, duel, 1, None, chk1, chk2),
        tp,
        LOCATION_MZONE,
        0,
        1,
        1,
        None,
    )?;
    let opt = g.get_first().is_some_and(|c| c.is_race(duel, RACE_PLANT));
    duel.send_to_grave(g, REASON_COST)?;
    if opt {
        e.set_label(duel, 1);
        e.set_category(duel, CATEGORY_DRAW + CATEGORY_HANDES);
        duel.set_operation_info(0, CATEGORY_DRAW, None, 0, tp, 2);
        duel.set_operation_info(0, CATEGORY_HANDES, None, 0, tp, 1);
    } else {
        e.set_label(duel, 2);
        e.set_category(duel, CATEGORY_TOHAND + CATEGORY_SEARCH);
        duel.set_operation_info(0, CATEGORY_TOHAND, None, 0, tp, LOCATION_DECK as i32);
    }
    Ok(false)
}

fn drop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    duel.hint(HINT_CARD, PLAYER_ALL, u64::from(ID));
    if duel.draw(tp, 2, REASON_EFFECT)? == 2 {
        duel.shuffle_hand(tp)?;
        duel.break_effect()?;
        duel.discard_hand(tp, aux::TRUE, 1, 1, REASON_EFFECT | REASON_DISCARD)?;
    }
    Ok(())
}

fn operation(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    match e.get_label(duel) {
        1 => {
            let c = e.get_handler(duel);
            let e1 = duel.create_effect(c);
            e1.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_CONTINUOUS);
            e1.set_code(duel, EVENT_PHASE + PHASE_END);
            e1.set_count_limit(duel, 1, 0);
            e1.set_reset(duel, RESET_PHASE | PHASE_END);
            e1.set_operation(duel, drop);
            duel.register_effect(e1, tp);
        }
        2 => {
            duel.hint(HINT_SELECTMSG, tp, HINTMSG_ATOHAND);
            let g = duel.select_matching_card(tp, thfilter, tp, LOCATION_DECK, 0, 1, 1, None)?;
            if !g.is_empty() {
                duel.send_to_hand(g, PLAYER_NONE, REASON_EFFECT)?;
                duel.confirm_cards(1 - tp, g)?;
            }
        }
        _ => {}
    }
    Ok(())
}
