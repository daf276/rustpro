// Performapal Monkeyboard
use crate::prelude::*;
use crate::testing::{Check, DECK, HAND};

const ID: u32 = 17330916;

pub const LISTED_SERIES: &[u16] = &[SET_PERFORMAPAL, SET_ODD_EYES];

// Copies in the hand to activate and discard, and Performapal Skullcrobat Joker and Odd-Eyes Synchron to lower and to search.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, HAND),
    (ID, HAND),
    (40318957, HAND),
    (82224646, HAND),
    (40318957, DECK),
    (82224646, DECK),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(0), Check::ResolvesEffect(aux::stringid(ID, 1))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    Pendulum::add_procedure_full(duel, c, false, None);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, 1160);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_range(duel, LOCATION_HAND);
    e1.set_cost(duel, reg);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_SINGLE);
    e2.set_code(duel, EFFECT_CHANGE_LSCALE);
    e2.set_property(duel, EFFECT_FLAG_SINGLE_RANGE);
    e2.set_range(duel, LOCATION_PZONE);
    e2.set_condition(duel, sccon);
    e2.set_value(duel, 4);
    c.register_effect(duel, e2);
    let e3 = e2.clone_effect(duel);
    e3.set_code(duel, EFFECT_CHANGE_RSCALE);
    c.register_effect(duel, e3);
    let e4 = duel.create_effect(c);
    e4.set_description(duel, aux::stringid(ID, 1));
    e4.set_type(duel, EFFECT_TYPE_IGNITION);
    e4.set_range(duel, LOCATION_HAND);
    e4.set_count_limit(duel, 1, 0);
    e4.set_cost(duel, Cost::self_discard);
    e4.set_target(duel, lvtg);
    e4.set_operation(duel, lvop);
    c.register_effect(duel, e4);
    let e5 = duel.create_effect(c);
    e5.set_category(duel, CATEGORY_TOHAND + CATEGORY_SEARCH);
    e5.set_type(duel, EFFECT_TYPE_IGNITION);
    e5.set_range(duel, LOCATION_PZONE);
    e5.set_count_limit(duel, 1, ID);
    e5.set_condition(duel, |duel, a| a.e.get_handler(duel).get_flag_effect(duel, ID) != 0);
    e5.set_target(duel, thtg);
    e5.set_operation(duel, thop);
    c.register_effect(duel, e5);
}

fn sccon(duel: &Duel, a: &Args) -> bool {
    let c = a.e.get_handler(duel);
    !duel.is_existing_matching_card(
        |c, duel| c.is_set_card(duel, &[SET_PERFORMAPAL]),
        a.e.get_handler_player(duel),
        LOCATION_PZONE,
        0,
        1,
        Some(c.into()),
    )
}

fn reg(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    if chk == 0 {
        return Ok(true);
    }
    let c = a.e.get_handler(duel);
    c.register_flag_effect(duel, ID, RESET_PHASE | PHASE_END, EFFECT_FLAG_OATH, 1);
    Ok(false)
}

fn thfilter(c: CardId, duel: &Duel) -> bool {
    c.is_level_below(duel, 4) && c.is_set_card(duel, &[SET_PERFORMAPAL]) && c.is_able_to_hand(duel)
}

fn thtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    stock::exists_target(duel, a, chk, thfilter, LOCATION_DECK, CATEGORY_TOHAND)
}

fn thop(duel: &mut Duel, a: &Args) -> Script<()> {
    stock::search(duel, a, thfilter, LOCATION_DECK)
}

fn filter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_PERFORMAPAL, SET_ODD_EYES]) && c.is_monster(duel) && c.is_level_above(duel, 2)
}

fn lvtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        let c = a.e.get_handler(duel);
        return Ok(duel.is_existing_matching_card(filter, a.tp, LOCATION_HAND, 0, 1, Some(c.into())));
    }
    Ok(false)
}

fn lvop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (c, tp) = (a.e.get_handler(duel), a.tp);
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_CONFIRM);
    let g = duel.select_matching_card(tp, filter, tp, LOCATION_HAND, 0, 1, 1, None)?;
    duel.confirm_cards(1 - tp, g)?;
    duel.shuffle_hand(tp)?;
    let Some(code) = g.get_first().map(|sc| sc.get_code(duel)) else {
        return Ok(());
    };
    let hg = duel.get_matching_group(|c, duel| c.is_code(duel, code), tp, LOCATION_HAND, 0, None);
    for tc in hg {
        let e1 = duel.create_effect(c);
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_code(duel, EFFECT_UPDATE_LEVEL);
        e1.set_value(duel, -1);
        e1.set_reset(duel, RESET_EVENT | (RESETS_STANDARD_PHASE_END & !RESET_TOFIELD));
        tc.register_effect(duel, e1);
    }
    Ok(())
}
