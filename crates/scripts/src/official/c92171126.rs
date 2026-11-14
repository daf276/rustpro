// Deftero Enneacraft - "alazoneIA"
use super::c28454232::{flip_quick, register_counter, register_set_summon};
use crate::prelude::*;
use crate::testing::{Check, DECK, HAND, MZONE, OPPONENT, PZONE};

const ID: u32 = 92171126;

pub const COUNTER_PLACE_LIST: &[u16] = &[COUNTER_EC];
pub const LISTED_SERIES: &[u16] = &[SET_ENNEACRAFT];

// In the hand, a Pendulum Zone, face-down on the field and in the Deck to reveal and search, with the opponent's Pot of
// Greed to answer.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, PZONE),
    (ID, MZONE),
    (ID, MZONE),
    (ID, DECK),
    (ID, DECK),
    (ID, DECK),
    (ID, DECK),
    (55144522, HAND | OPPONENT),
    (55144522, HAND | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
    Check::ResolvesEffect(aux::stringid(ID, 2)),
    Check::ResolvesEffect(aux::stringid(ID, 3)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    register_counter(duel, c);
    register_reveal::<ID>(duel, c);
    register_set_summon::<ID>(duel, c);
    let e4 = flip_quick::<ID>(duel, c, CATEGORY_TOHAND + CATEGORY_SEARCH, 0);
    e4.set_condition(duel, thcon);
    e4.set_target(duel, thtg);
    e4.set_operation(duel, thop);
    c.register_effect(duel, e4);
    let e5 = duel.create_effect(c);
    e5.set_description(duel, aux::stringid(ID, 3));
    e5.set_category(duel, CATEGORY_REMOVE);
    e5.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_F);
    e5.set_code(duel, EVENT_FLIP);
    e5.set_target(duel, rmtg);
    e5.set_operation(duel, rmop);
    c.register_effect(duel, e5);
}

// The reveal-and-pick search from the Pendulum Zone of the Level 1 Enneacraft monsters.
pub fn register_reveal<const ID: u32>(duel: &mut Duel, c: CardId) {
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 0));
    e2.set_category(duel, CATEGORY_TOHAND + CATEGORY_SEARCH);
    e2.set_type(duel, EFFECT_TYPE_IGNITION);
    e2.set_range(duel, LOCATION_PZONE);
    e2.set_count_limit(duel, 1, ID);
    e2.set_cost(duel, Cost::pay_lp::<900>);
    e2.set_target(duel, rvtg);
    e2.set_operation(duel, rvop);
    c.register_effect(duel, e2);
}

fn rvfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_ENNEACRAFT]) && c.is_able_to_hand(duel) && !c.is_public(duel)
}

fn rvtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(rvfilter, a.tp, LOCATION_DECK, 0, 3, None));
    }
    duel.set_operation_info(0, CATEGORY_TOHAND, None, 1, a.tp, LOCATION_DECK as i32);
    Ok(false)
}

fn rvop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_CONFIRM);
    let g = duel.select_matching_card(tp, rvfilter, tp, LOCATION_DECK, 0, 3, 3, None)?;
    if g.len() != 3 {
        return Ok(());
    }
    duel.confirm_cards(1 - tp, g)?;
    let sg = g.random_select(duel, 1 - tp, 1);
    duel.shuffle_deck(tp)?;
    if !sg.is_empty() {
        duel.disable_shuffle_check(true);
        duel.send_to_hand(sg, PLAYER_NONE, REASON_EFFECT)?;
    }
    Ok(())
}

fn thcon(duel: &Duel, a: &Args) -> bool {
    let (tp, ev) = (a.tp, a.ev as u8);
    if a.ep == tp || a.e.get_handler(duel).is_faceup(duel) {
        return false;
    }
    if a.re
        .is_some_and(|re| re.is_has_category(duel, CATEGORY_SEARCH) || re.is_has_category(duel, CATEGORY_DRAW))
    {
        return true;
    }
    let infos = [
        duel.get_operation_info(ev, CATEGORY_TOHAND),
        duel.get_possible_operation_info(ev, CATEGORY_TOHAND),
    ];
    let loc = infos.iter().flatten().fold(0, |loc, info| loc | info.op_param);
    let g = infos
        .iter()
        .flatten()
        .filter_map(|info| info.op_cards)
        .fold(CardSet::default(), |g, cards| g.union(cards));
    loc & LOCATION_DECK as i32 > 0 || (!g.is_empty() && g.is_exists(duel, |c, duel| c.is_location(duel, LOCATION_DECK), 1, None))
}

pub fn thfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_ENNEACRAFT]) && c.is_able_to_hand(duel)
}

pub fn thtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    stock::exists_target(duel, a, chk, thfilter, LOCATION_DECK, CATEGORY_TOHAND)
}

pub fn thop(duel: &mut Duel, a: &Args) -> Script<()> {
    stock::search(duel, a, thfilter, LOCATION_DECK)
}

fn rmtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(true);
    }
    duel.set_operation_info(0, CATEGORY_REMOVE, None, 1, 1 - a.tp, LOCATION_HAND as i32);
    Ok(false)
}

struct ReturnToHand;

impl aux::DelayedOperation for ReturnToHand {
    fn operate(duel: &mut Duel, ag: CardSet, _: &Args) -> Script<()> {
        duel.send_to_hand(ag, PLAYER_NONE, REASON_EFFECT).map(drop)
    }
}

fn rmop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    let g = duel.get_matching_group(CardId::is_able_to_remove, tp, 0, LOCATION_HAND, None);
    if g.is_empty() {
        return Ok(());
    }
    let rg = g.random_select(duel, tp, 1);
    if !rg.is_empty() {
        aux::remove_until::<ReturnToHand>(duel, rg, POS_FACEUP, REASON_EFFECT | REASON_TEMPORARY, PHASE_END, ID, a.e, tp)?;
    }
    Ok(())
}
