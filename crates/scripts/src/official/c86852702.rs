// Machina Redeployment
use crate::prelude::*;
use crate::testing::{Check, DECK, HAND};

const ID: u32 = 86852702;

pub const LISTED_NAMES: &[u32] = &[ID];
pub const LISTED_SERIES: &[u16] = &[SET_MACHINA];
// Machina Gearframe in hand to discard, with Machina monsters and Machina Overdrive in the Deck to add.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (42940404, HAND),
    (42940404, DECK),
    (5556499, DECK),
    (46033517, DECK),
    (59741415, DECK),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_TOHAND + CATEGORY_SEARCH);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_count_limit_full(duel, 1, ID, EFFECT_COUNT_CODE_OATH);
    e1.set_cost(duel, cost::<1>);
    e1.set_target(duel, target::<1>);
    e1.set_operation(duel, activate::<1>);
    c.register_effect(duel, e1);
    let e2 = e1.clone_effect(duel);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_cost(duel, cost::<2>);
    e2.set_target(duel, target::<2>);
    e2.set_operation(duel, activate::<2>);
    c.register_effect(duel, e2);
}

// s.cfilter1 for option 1, s.cfilter2 for option 2.
fn cfilter<const OPTION: u8>(c: CardId, duel: &Duel) -> bool {
    c.is_discardable(duel) && (OPTION == 1 || c.is_set_card(duel, &[SET_MACHINA]))
}

// s.filter1 for option 1, s.filter2 for option 2.
fn filter<const OPTION: u8>(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_MACHINA]) && (if OPTION == 1 { c.is_monster(duel) } else { !c.is_code(duel, ID) }) && c.is_able_to_hand(duel)
}

fn cost<const OPTION: u8>(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        let handler = a.e.get_handler(duel);
        return Ok(duel.is_existing_matching_card(cfilter::<OPTION>, tp, LOCATION_HAND, 0, 1, Some(handler.into())));
    }
    duel.discard_hand(tp, cfilter::<OPTION>, 1, 1, REASON_COST | REASON_DISCARD)?;
    Ok(false)
}

fn target<const OPTION: u8>(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let g = duel.get_matching_group(filter::<OPTION>, a.tp, LOCATION_DECK, 0, None);
    if chk == 0 {
        return Ok(aux::select_unselect_group_check(duel, g, 2, 2, &aux::dncheck));
    }
    duel.set_operation_info(0, CATEGORY_TOHAND, Some(g), 2, 0, 0);
    Ok(false)
}

fn activate<const OPTION: u8>(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    let g = duel.get_matching_group(filter::<OPTION>, tp, LOCATION_DECK, 0, None);
    let sg = aux::select_unselect_group(duel, g, 2, 2, &aux::dncheck, tp, HINTMSG_ATOHAND)?;
    if sg.len() == 2 {
        duel.send_to_hand(sg, PLAYER_NONE, REASON_EFFECT)?;
        duel.confirm_cards(1 - tp, sg)?;
    }
    Ok(())
}
