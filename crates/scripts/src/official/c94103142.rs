// Radiant Typhoon Manifestation
use super::c20508881::add_set_effect;
use crate::prelude::*;
use crate::testing::{Check, DECK, SZONE};

const ID: u32 = 94103142;

pub const LISTED_SERIES: &[u16] = &[SET_RADIANT_TYPHOON];
pub const LISTED_NAMES: &[u32] = &[CARD_MYSTICAL_SPACE_TYPHOON];
// Mystical Space Typhoon in the Deck to search and set to destroy this card with, and Radiant Typhoon Swen to send.
pub const SUPPORT: &[(u32, u16)] = &[
    (CARD_MYSTICAL_SPACE_TYPHOON, DECK),
    (CARD_MYSTICAL_SPACE_TYPHOON, SZONE),
    (80538047, DECK),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_target(duel, efftg);
    e1.set_operation(duel, effop);
    e1.set_hint_timing(duel, 0, TIMING_STANDBY_PHASE | TIMING_MAIN_END | TIMINGS_CHECK_MONSTER_E);
    c.register_effect(duel, e1);
    add_set_effect(duel, c, aux::stringid(ID, 1));
}

fn tgfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_RADIANT_TYPHOON]) && c.is_monster(duel) && c.is_able_to_grave(duel)
}

fn thfilter(c: CardId, duel: &Duel) -> bool {
    c.is_code(duel, CARD_MYSTICAL_SPACE_TYPHOON) && c.is_able_to_hand(duel)
}

fn efftg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let b1 = !duel.has_flag_effect(tp, ID) && duel.is_existing_matching_card(tgfilter, tp, LOCATION_DECK, 0, 1, None);
    let b2 = !duel.has_flag_effect(tp, ID + 1) && duel.is_existing_matching_card(thfilter, tp, LOCATION_DECK | LOCATION_GRAVE, 0, 1, None);
    if chk == 0 {
        return Ok(b1 || b2);
    }
    let op = duel.select_effect(tp, &[(b1, aux::stringid(ID, 2)), (b2, aux::stringid(ID, 3))])?;
    e.set_label(duel, op.map_or(0, |op| op as i64));
    if op == Some(1) {
        e.set_category(duel, CATEGORY_TOGRAVE);
        duel.register_flag_effect(tp, ID, RESET_PHASE | PHASE_END, 0, 1);
        duel.set_operation_info(0, CATEGORY_TOGRAVE, None, 1, tp, LOCATION_DECK as i32);
    } else if op == Some(2) {
        e.set_category(duel, CATEGORY_SEARCH + CATEGORY_TOHAND);
        duel.register_flag_effect(tp, ID + 1, RESET_PHASE | PHASE_END, 0, 1);
        duel.set_operation_info(0, CATEGORY_TOHAND, None, 1, tp, (LOCATION_DECK | LOCATION_GRAVE) as i32);
    }
    Ok(false)
}

fn effop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    match a.e.get_label(duel) {
        1 => {
            duel.hint(HINT_SELECTMSG, tp, HINTMSG_TOGRAVE);
            let g = duel.select_matching_card(tp, tgfilter, tp, LOCATION_DECK, 0, 1, 1, None)?;
            if !g.is_empty() {
                duel.send_to_grave(g, REASON_EFFECT)?;
            }
        }
        2 => {
            duel.hint(HINT_SELECTMSG, tp, HINTMSG_ATOHAND);
            let filter = aux::necro_valley_filter(thfilter);
            let g = duel.select_matching_card(tp, filter, tp, LOCATION_DECK | LOCATION_GRAVE, 0, 1, 1, None)?;
            if !g.is_empty() {
                duel.send_to_hand(g, PLAYER_NONE, REASON_EFFECT)?;
                duel.confirm_cards(1 - tp, g)?;
            }
        }
        _ => {}
    }
    Ok(())
}
