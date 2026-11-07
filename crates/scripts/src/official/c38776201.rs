// Sacred Beasts Released
use crate::prelude::*;
use crate::testing::{Check, DECK, GRAVE, HAND};

const ID: u32 = 38776201;

// Differently named Sacred Beast monsters in the Deck, listed twice since the opening hand draws some, a copy in the hand to
// activate and one in the Graveyard.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, GRAVE),
    (23856331, DECK),
    (50251045, DECK),
    (96345184, DECK),
    (59138498, DECK),
    (23856331, DECK),
    (50251045, DECK),
    (96345184, DECK),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_TOHAND + CATEGORY_SEARCH + CATEGORY_HANDES);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_count_limit(duel, 1, (ID, 0));
    e1.set_target(duel, th3tg);
    e1.set_operation(duel, th3op);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_TOHAND + CATEGORY_SEARCH);
    e2.set_type(duel, EFFECT_TYPE_IGNITION);
    e2.set_range(duel, LOCATION_GRAVE);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_condition(duel, aux::exccon);
    e2.set_cost(duel, Cost::self_banish);
    e2.set_target(duel, th1tg);
    e2.set_operation(duel, th1op);
    c.register_effect(duel, e2);
}

fn th3filter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_SACRED_BEAST]) && c.is_monster(duel) && c.is_able_to_hand(duel)
}

fn th3tg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        let g = duel.get_matching_group(th3filter, tp, LOCATION_DECK, 0, None);
        return Ok(aux::select_unselect_group_check(duel, g, 3, 3, &aux::dncheck));
    }
    duel.set_operation_info(0, CATEGORY_TOHAND, None, 3, tp, LOCATION_DECK as i32);
    duel.set_operation_info(0, CATEGORY_HANDES, None, 0, tp, 2);
    Ok(false)
}

fn th3op(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    let g = duel.get_matching_group(th3filter, tp, LOCATION_DECK, 0, None);
    if g.len() < 3 {
        return Ok(());
    }
    let sg = aux::select_unselect_group(duel, g, 3, 3, &aux::dncheck, tp, HINTMSG_ATOHAND)?;
    if sg.len() == 3 && duel.send_to_hand(sg, PLAYER_NONE, REASON_EFFECT)? > 0 {
        duel.confirm_cards(1 - tp, sg)?;
        duel.shuffle_hand(tp)?;
        duel.break_effect()?;
        duel.discard_hand(tp, aux::TRUE, 2, 2, REASON_EFFECT | REASON_DISCARD)?;
    }
    Ok(())
}

fn th1filter(c: CardId, duel: &Duel) -> bool {
    c.is_level(duel, &[10])
        && c.is_race(duel, RACE_PYRO | RACE_THUNDER | RACE_FIEND)
        && !c.is_summonable_card(duel)
        && c.is_able_to_hand(duel)
}

fn th1tg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(th1filter, tp, LOCATION_DECK, 0, 1, None));
    }
    duel.set_operation_info(0, CATEGORY_TOHAND, None, 1, tp, LOCATION_DECK as i32);
    Ok(false)
}

fn th1op(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_ATOHAND);
    let g = duel.select_matching_card(tp, th1filter, tp, LOCATION_DECK, 0, 1, 1, None)?;
    if !g.is_empty() && duel.send_to_hand(g, PLAYER_NONE, REASON_EFFECT)? > 0 {
        duel.confirm_cards(1 - tp, g)?;
    }
    Ok(())
}
