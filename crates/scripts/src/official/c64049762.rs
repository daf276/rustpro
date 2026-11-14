// Blitzclique - Breakaway
use crate::prelude::*;
use crate::testing::{Check, DECK, GRAVE, HAND, MZONE};

const ID: u32 = 64049762;

pub const LISTED_SERIES: &[u16] = &[SET_BLITZCLIQUE];

// The field of its chain data table.
const CHOICE: u8 = 0;

// A copy in the GY, a Thunder Normal Monster (LaLa Li-oon) on the field and Surge Blitzclique in the hand to destroy a card
// with, and a Blitzclique Continuous Trap in the Deck.
pub const SUPPORT: &[(u32, u16)] = &[(ID, GRAVE), (9430387, MZONE), (22912101, HAND), (23526128, DECK)];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_count_limit(duel, 1, (ID, 0));
    e1.set_cost(duel, effcost);
    e1.set_target(duel, efftg);
    e1.set_operation(duel, effop);
    e1.set_hint_timing(duel, 0, TIMING_STANDBY_PHASE | TIMING_MAIN_END | TIMINGS_CHECK_MONSTER_E);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_TOHAND);
    e2.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e2.set_property_full(duel, EFFECT_FLAG_DELAY, EFFECT_FLAG2_CHECK_SIMULTANEOUS);
    e2.set_code(duel, EVENT_DESTROYED);
    e2.set_range(duel, LOCATION_GRAVE);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_condition(duel, selfthcon);
    e2.set_target(duel, selfthtg);
    e2.set_operation(duel, selfthop);
    c.register_effect(duel, e2);
}

fn plfilter(c: CardId, duel: &Duel, tp: u8) -> bool {
    c.is_set_card(duel, &[SET_BLITZCLIQUE]) && c.is_continuous_trap(duel) && !c.is_forbidden(duel) && c.check_unique_on_field(duel, tp)
}

fn descostfilter(c: CardId, duel: &Duel, hc: CardId) -> bool {
    let excluded = c.get_equip_group(duel).union(c.into()).union(hc.into());
    c.is_race(duel, RACE_THUNDER)
        && c.is_faceup(duel)
        && c.is_able_to_hand_as_cost(duel)
        && duel.is_existing_target(aux::TRUE, 0, LOCATION_ONFIELD, LOCATION_ONFIELD, 1, Some(excluded))
}

fn can_place_trap(duel: &Duel, c: CardId, tp: u8) -> bool {
    let ft = duel.get_location_count(tp, LOCATION_SZONE) - i32::from(c.is_location(duel, LOCATION_HAND));
    ft > 0 && duel.is_existing_matching_card(|c, duel| plfilter(c, duel, tp), tp, LOCATION_DECK, 0, 1, None)
}

fn effcost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    let b1 = can_place_trap(duel, c, tp);
    let b2 = duel.is_existing_matching_card(|tc, duel| descostfilter(tc, duel, c), tp, LOCATION_MZONE, 0, 1, None);
    if chk == 0 {
        return Ok(b1 || b2);
    }
    let choice = duel.select_effect(tp, &[(b1, aux::stringid(ID, 2)), (b2, aux::stringid(ID, 3))])?;
    if let Some(choice) = choice {
        e.set_chain_data(duel, CHOICE, choice as i64);
    }
    if choice == Some(2) {
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_RTOHAND);
        let g = duel.select_matching_card(tp, |tc, duel| descostfilter(tc, duel, c), tp, LOCATION_MZONE, 0, 1, 1, None)?;
        duel.hint_selection(g);
        duel.send_to_hand(g, PLAYER_NONE, REASON_COST)?;
    }
    Ok(false)
}

fn efftg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if let Some(chkc) = chkc {
        return Ok(duel.get_chain_info::<i32>(0, CHAININFO_TARGET_PARAM) == 2 && chkc.is_on_field(duel) && chkc != c);
    }
    let b1 = can_place_trap(duel, c, tp);
    let b2 = duel.is_existing_target(aux::TRUE, tp, LOCATION_ONFIELD, LOCATION_ONFIELD, 1, Some(c.into()));
    if chk == 0 {
        return Ok(b1 || b2);
    }
    let choice = match e.get_chain_data(duel, CHOICE) {
        Some(choice) => Some(choice as usize),
        None => duel.select_effect(tp, &[(b1, aux::stringid(ID, 2)), (b2, aux::stringid(ID, 3))])?,
    };
    if let Some(choice) = choice {
        e.set_chain_data(duel, CHOICE, choice as i64);
    }
    if choice == Some(1) {
        e.set_category(duel, 0);
        e.set_property(duel, 0);
    } else if choice == Some(2) {
        e.set_category(duel, CATEGORY_DESTROY);
        e.set_property(duel, EFFECT_FLAG_CARD_TARGET);
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_DESTROY);
        let g = duel.select_target(tp, aux::TRUE, tp, LOCATION_ONFIELD, LOCATION_ONFIELD, 1, 1, Some(c.into()))?;
        duel.set_operation_info(0, CATEGORY_DESTROY, g, 1, tp, 0);
    }
    Ok(false)
}

fn effop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    match e.get_chain_data(duel, CHOICE) {
        Some(1) => {
            if duel.get_location_count(tp, LOCATION_SZONE) <= 0 {
                return Ok(());
            }
            duel.hint(HINT_SELECTMSG, tp, HINTMSG_TOFIELD);
            let g = duel.select_matching_card(tp, |c, duel| plfilter(c, duel, tp), tp, LOCATION_DECK, 0, 1, 1, None)?;
            if let Some(sc) = g.get_first() {
                duel.move_to_field(sc, tp, tp, LOCATION_SZONE, POS_FACEUP, true)?;
            }
        }
        Some(2) => {
            if let Some(tc) = duel.get_first_target()
                && tc.is_relate_to_effect(duel, e)
            {
                duel.destroy(tc, REASON_EFFECT)?;
            }
        }
        _ => {}
    }
    Ok(())
}

// The condition, target and operation of the "add this card to your hand" effect from the GY, which Blitzclique - Steppleader
// shares.
pub fn selfthcon(duel: &Duel, a: &Args) -> bool {
    let Some(re) = a.re else { return false };
    a.rp == a.tp
        && !a.eg.is_some_and(|eg| eg.is_contains(a.e.get_handler(duel)))
        && a.eg
            .is_some_and(|eg| eg.is_exists(duel, |c, duel| c.is_reason(duel, REASON_EFFECT), 1, None))
        && re.get_handler(duel).is_set_card(duel, &[SET_BLITZCLIQUE])
}

pub fn selfthtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let c = a.e.get_handler(duel);
    if chk == 0 {
        return Ok(c.is_able_to_hand(duel));
    }
    duel.set_operation_info(0, CATEGORY_TOHAND, Some(c.into()), 1, a.tp, 0);
    Ok(false)
}

pub fn selfthop(duel: &mut Duel, a: &Args) -> Script<()> {
    let c = a.e.get_handler(duel);
    if c.is_relate_to_effect(duel, a.e) {
        duel.send_to_hand(c, PLAYER_NONE, REASON_EFFECT)?;
    }
    Ok(())
}
