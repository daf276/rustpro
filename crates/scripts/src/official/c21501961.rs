// Pair Bear Scare!!
use crate::prelude::*;
use crate::testing::{Check, DECK, HAND, MZONE, OPPONENT};

const ID: u32 = 21501961;

pub const LISTED_NAMES: &[u32] = &[ID];

// In the hand to activate, with copies in the opponent's Deck and hand to reveal and a Sangan of theirs to destroy.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, DECK | OPPONENT),
    (ID, DECK | OPPONENT),
    (ID, HAND | OPPONENT),
    (26202165, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_RECOVER + CATEGORY_DESTROY);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_count_limit_full(duel, 1, (ID, 0), EFFECT_COUNT_CODE_OATH);
    e1.set_target(duel, target);
    e1.set_operation(duel, activate);
    e1.set_hint_timing(duel, 0, TIMING_STANDBY_PHASE | TIMING_MAIN_END | TIMINGS_CHECK_MONSTER_E);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_TOHAND);
    e2.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_F);
    e2.set_code(duel, EVENT_TO_GRAVE);
    e2.set_count_limit_full(duel, 1, (ID, 1), EFFECT_COUNT_CODE_DUEL);
    e2.set_condition(duel, |duel, a| a.e.get_handler(duel).is_previous_location(duel, LOCATION_ONFIELD));
    e2.set_target(duel, thtg);
    e2.set_operation(duel, thop);
    c.register_effect(duel, e2);
}

fn target(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(true);
    }
    duel.set_possible_operation_info(0, CATEGORY_RECOVER, None, 0, PLAYER_ALL, 2000);
    duel.set_possible_operation_info(0, CATEGORY_DESTROY, None, 1, 1 - a.tp, LOCATION_MZONE as i32);
    Ok(false)
}

fn revealfilter(c: CardId, duel: &Duel) -> bool {
    c.is_code(duel, ID) && !c.is_public(duel)
}

fn activate(duel: &mut Duel, a: &Args) -> Script<()> {
    let (tp, opp) = (a.tp, 1 - a.tp);
    let location = LOCATION_HAND | LOCATION_DECK;
    if duel.is_existing_matching_card(revealfilter, opp, location, 0, 1, None) && duel.select_yes_no(opp, aux::stringid(ID, 2))? {
        duel.hint(HINT_SELECTMSG, opp, HINTMSG_CONFIRM);
        let g = duel.select_matching_card(opp, revealfilter, opp, location, 0, 1, 1, None)?;
        if let Some(sc) = g.get_first() {
            duel.confirm_cards(tp, sc)?;
            if sc.is_location(duel, LOCATION_HAND) {
                duel.shuffle_hand(opp)?;
            } else {
                duel.shuffle_deck(opp)?;
            }
        }
        duel.recover(tp, 2000, REASON_EFFECT)?;
        duel.recover(opp, 2000, REASON_EFFECT)?;
    } else {
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_DESTROY);
        let g = duel.select_matching_card(tp, aux::TRUE, tp, 0, LOCATION_MZONE, 1, 1, None)?;
        if !g.is_empty() {
            duel.hint_selection(g);
            duel.destroy(g, REASON_EFFECT)?;
        }
    }
    Ok(())
}

fn thtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(true);
    }
    let c = a.e.get_handler(duel);
    duel.set_operation_info(0, CATEGORY_TOHAND, Some(c.into()), 1, a.tp, 0);
    Ok(false)
}

fn thop(duel: &mut Duel, a: &Args) -> Script<()> {
    let c = a.e.get_handler(duel);
    if c.is_relate_to_effect(duel, a.e) {
        duel.send_to_hand(c, 1 - a.tp, REASON_EFFECT)?;
    }
    Ok(())
}
