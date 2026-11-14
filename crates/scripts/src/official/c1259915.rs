// Sacred Beasts Thunderclap
use crate::prelude::*;
use crate::testing::{Check, DECK, GRAVE, HAND, SZONE};

const ID: u32 = 1259915;

pub const LISTED_SERIES: &[u16] = &[SET_SACRED_BEAST];

const FALLEN_PARADISE: u32 = 65861210;

pub const LISTED_NAMES: &[u32] = &[ID, FALLEN_PARADISE];

// One copy on the field and two more in the Deck and Graveyard to place, a Level 10 Sacred Beast monster in the hand to
// reveal and Fallen Paradise in the Deck.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, SZONE),
    (ID, DECK),
    (ID, DECK),
    (ID, GRAVE),
    (23856331, HAND),
    (FALLEN_PARADISE, DECK),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e0 = duel.create_effect(c);
    e0.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e0.set_code(duel, EVENT_FREE_CHAIN);
    c.register_effect(duel, e0);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_range(duel, LOCATION_SZONE);
    e1.set_count_limit(duel, 1, (ID, 0));
    e1.set_target(duel, pltg);
    e1.set_operation(duel, plop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_TOHAND);
    e2.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e2.set_code(duel, EVENT_PHASE + PHASE_END);
    e2.set_range(duel, LOCATION_GRAVE);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_condition(duel, |duel, a| duel.is_turn_player(1 - a.tp));
    e2.set_target(duel, thtg);
    e2.set_operation(duel, stock::self_to_hand);
    c.register_effect(duel, e2);
}

fn selfplfilter(c: CardId, duel: &Duel, tp: u8) -> bool {
    c.is_code(duel, ID) && !c.is_forbidden(duel) && c.check_unique_on_field(duel, tp)
}

fn pltg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.get_location_count(tp, LOCATION_SZONE) >= 2
            && duel.is_existing_matching_card(
                |c, duel| selfplfilter(c, duel, tp),
                tp,
                LOCATION_HAND | LOCATION_DECK | LOCATION_GRAVE,
                0,
                2,
                None,
            ));
    }
    Ok(false)
}

fn revealfilter(c: CardId, duel: &Duel) -> bool {
    c.is_level(duel, &[10]) && c.is_set_card(duel, &[SET_SACRED_BEAST]) && !c.is_public(duel)
}

fn fieldplfilter(c: CardId, duel: &Duel, tp: u8) -> bool {
    c.is_code(duel, FALLEN_PARADISE) && !c.is_forbidden(duel) && c.check_unique_on_field(duel, tp)
}

fn plop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    if duel.get_location_count(tp, LOCATION_SZONE) < 2 {
        return Ok(());
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TOFIELD);
    let filter = aux::necro_valley_filter(|c, duel| selfplfilter(c, duel, tp));
    let g = duel.select_matching_card(tp, filter, tp, LOCATION_HAND | LOCATION_DECK | LOCATION_GRAVE, 0, 2, 2, None)?;
    if g.len() != 2 {
        return Ok(());
    }
    for tc in g {
        duel.move_to_field(tc, tp, tp, LOCATION_SZONE, POS_FACEUP, true)?;
    }
    if duel.is_existing_matching_card(revealfilter, tp, LOCATION_HAND, 0, 1, None)
        && duel.is_existing_matching_card(|c, duel| fieldplfilter(c, duel, tp), tp, LOCATION_DECK, 0, 1, None)
        && duel.select_yes_no(tp, aux::stringid(ID, 2))?
    {
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_CONFIRM);
        let rg = duel.select_matching_card(tp, revealfilter, tp, LOCATION_HAND, 0, 1, 1, None)?;
        if rg.is_empty() {
            return Ok(());
        }
        duel.break_effect()?;
        duel.confirm_cards(1 - tp, rg)?;
        duel.shuffle_hand(tp)?;
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_TOFIELD);
        let g = duel.select_matching_card(tp, |c, duel| fieldplfilter(c, duel, tp), tp, LOCATION_DECK, 0, 1, 1, None)?;
        if let Some(sc) = g.get_first() {
            duel.break_effect()?;
            duel.place_field_spell(sc, tp)?;
        }
    }
    Ok(())
}

fn thtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let c = a.e.get_handler(duel);
    if chk == 0 {
        return Ok(c.is_able_to_hand(duel));
    }
    duel.set_operation_info(0, CATEGORY_TOHAND, Some(c.into()), 1, a.tp, 0);
    Ok(false)
}
