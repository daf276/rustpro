// Shimmering Shatterer of the Sacred Beasts
use engine::constants::MSG_CHAIN_DISABLED;

use crate::prelude::*;
use crate::testing::{Check, DECK, GRAVE, HAND, MZONE, OPPONENT, REMOVED};

const ID: u32 = 3441553;
const CARD_ILLUSION_GATE: u32 = 33017964;
const CHOICE: u8 = 0;
const EVENT_VALUE: u8 = 1;

pub const LISTED_NAMES: &[u32] = &[CARD_ILLUSION_GATE];

// In the hand with Uria (Level 10) to reveal, Illusion Gate in the Deck to add, Sacred Beast Traps in the Graveyard and banishment
// to shuffle, and an opponent's monster to destroy; Uria on the field with Monster Reborn in the opponent's hand to negate.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, HAND),
    (23856331, HAND),
    (23856331, MZONE),
    (CARD_ILLUSION_GATE, DECK),
    (CARD_ILLUSION_GATE, DECK),
    (50147815, GRAVE),
    (50147815, GRAVE),
    (86132414, REMOVED),
    (86132414, GRAVE),
    (26202165, MZONE | OPPONENT),
    (83764718, HAND | OPPONENT),
    (83764718, HAND | OPPONENT),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0)), Check::Message(MSG_CHAIN_DISABLED)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_cost(duel, effcost);
    e1.set_target(duel, efftg);
    e1.set_operation(duel, effop);
    e1.set_count_limit_full(duel, 1, ID, EFFECT_COUNT_CODE_OATH);
    e1.set_hint_timing(duel, 0, TIMING_STANDBY_PHASE | TIMING_MAIN_END | TIMINGS_CHECK_MONSTER_E);
    c.register_effect(duel, e1);
}

fn revealfilter(c: CardId, duel: &Duel) -> bool {
    c.is_level(duel, &[10]) && c.is_set_card(duel, &[SET_SACRED_BEAST]) && !c.is_public(duel)
}

fn thfilter(c: CardId, duel: &Duel) -> bool {
    c.is_code(duel, CARD_ILLUSION_GATE) && c.is_able_to_hand(duel)
}

fn tdfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_SACRED_BEAST]) && c.is_trap(duel) && c.is_faceup(duel) && c.is_able_to_deck_as_cost(duel)
}

fn discardable(c: CardId, duel: &Duel) -> bool {
    c.is_discardable_full(duel, REASON_EFFECT, None)
}

fn effcost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let event = duel.check_event_full(EVENT_CHAINING);
    let option_1 = event.is_some_and(|event| {
        let previous = (event.event_value as u8).wrapping_sub(1);
        event.event_player == 1 - tp
            && chain::is_triggering_player(duel, previous, tp)
            && chain::is_triggering_setcode(duel, previous, &[SET_SACRED_BEAST])
            && chain::is_triggering_type(duel, previous, TYPE_MONSTER)
            && duel.is_chain_disablable(event.event_value as u8)
    });
    let option_2 = duel.is_existing_matching_card(revealfilter, tp, LOCATION_HAND, 0, 1, None)
        && duel.is_existing_matching_card(discardable, tp, LOCATION_HAND, 0, 1, None)
        && duel.is_existing_matching_card(thfilter, tp, LOCATION_DECK, 0, 1, None);
    let option_3 = duel.get_matching_group_count(tdfilter, tp, LOCATION_GRAVE | LOCATION_REMOVED, 0, None) >= 3
        && duel.is_existing_target(aux::TRUE, tp, 0, LOCATION_ONFIELD, 1, None);
    if chk == 0 {
        return Ok(option_1 || option_2 || option_3);
    }
    let choice = duel.select_effect(
        tp,
        &[
            (option_1, aux::stringid(ID, 1)),
            (option_2, aux::stringid(ID, 2)),
            (option_3, aux::stringid(ID, 3)),
        ],
    )?;
    if let Some(choice) = choice {
        e.set_chain_data(duel, CHOICE, choice as i64);
    }
    match choice {
        Some(1) => {
            e.set_category(duel, CATEGORY_DISABLE);
            e.set_property(duel, 0);
            if let Some(event) = event {
                e.set_chain_data(duel, EVENT_VALUE, i64::from(event.event_value));
            }
        }
        Some(2) => {
            e.set_category(duel, CATEGORY_HANDES + CATEGORY_TOHAND + CATEGORY_SEARCH);
            e.set_property(duel, 0);
            duel.hint(HINT_SELECTMSG, tp, HINTMSG_CONFIRM);
            let g = duel.select_matching_card(tp, revealfilter, tp, LOCATION_HAND, 0, 1, 1, None)?;
            duel.confirm_cards(1 - tp, g)?;
            duel.shuffle_hand(tp)?;
        }
        Some(3) => {
            e.set_category(duel, CATEGORY_DESTROY);
            e.set_property(duel, EFFECT_FLAG_CARD_TARGET);
            duel.hint(HINT_SELECTMSG, tp, HINTMSG_TODECK);
            let g = duel.select_matching_card(tp, tdfilter, tp, LOCATION_GRAVE | LOCATION_REMOVED, 0, 3, 3, None)?;
            duel.hint_selection(g);
            duel.send_to_deck(g, PLAYER_NONE, SEQ_DECKSHUFFLE, REASON_COST)?;
        }
        _ => {}
    }
    Ok(false)
}

fn efftg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if let Some(chkc) = chkc {
        return Ok(e.get_chain_data(duel, CHOICE) == Some(3) && chkc.is_controler(duel, 1 - tp) && chkc.is_on_field(duel));
    }
    if chk == 0 {
        return Ok(true);
    }
    match e.get_chain_data(duel, CHOICE) {
        Some(1) => {
            duel.set_operation_info(0, CATEGORY_DISABLE, a.eg, 1, tp, 0);
        }
        Some(2) => {
            duel.set_operation_info(0, CATEGORY_HANDES, None, 0, tp, 1);
            duel.set_operation_info(0, CATEGORY_TOHAND, None, 1, tp, LOCATION_DECK as i32);
        }
        Some(3) => {
            duel.hint(HINT_SELECTMSG, tp, HINTMSG_DESTROY);
            let g = duel.select_target(tp, aux::TRUE, tp, 0, LOCATION_ONFIELD, 1, 1, None)?;
            duel.set_operation_info(0, CATEGORY_DESTROY, g, 1, tp, 0);
        }
        _ => {}
    }
    Ok(false)
}

fn effop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    match e.get_chain_data(duel, CHOICE) {
        Some(1) => {
            if let Some(link) = e.get_chain_data(duel, EVENT_VALUE) {
                duel.negate_effect(link as u8);
            }
        }
        Some(2) => {
            if duel.discard_hand(tp, discardable, 1, 1, REASON_EFFECT | REASON_DISCARD)? > 0 {
                duel.hint(HINT_SELECTMSG, tp, HINTMSG_ATOHAND);
                let g = duel.select_matching_card(tp, thfilter, tp, LOCATION_DECK, 0, 1, 1, None)?;
                if !g.is_empty() {
                    duel.send_to_hand(g, PLAYER_NONE, REASON_EFFECT)?;
                    duel.confirm_cards(1 - tp, g)?;
                }
            }
        }
        Some(3) => {
            if let Some(tc) = duel.get_first_target().filter(|tc| tc.is_relate_to_effect(duel, e)) {
                duel.destroy(tc, REASON_EFFECT)?;
            }
        }
        _ => {}
    }
    Ok(())
}
