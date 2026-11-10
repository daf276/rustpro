// Soul Servant
use crate::prelude::*;
use crate::testing::{Check, DECK, GRAVE, HAND};

const ID: u32 = 23020408;

pub const LISTED_NAMES: &[u32] = &[CARD_DARK_MAGICIAN, CARD_DARK_MAGICIAN_GIRL, ID];

// In hand with Dark Magician in the Deck to place, and in the Graveyard with both Dark Magicians there to draw.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, GRAVE),
    (CARD_DARK_MAGICIAN, DECK),
    (CARD_DARK_MAGICIAN, DECK),
    (CARD_DARK_MAGICIAN, GRAVE),
    (CARD_DARK_MAGICIAN_GIRL, GRAVE),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_TODECK);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_hint_timing(duel, TIMING_END_PHASE, TIMING_END_PHASE);
    e1.set_target(duel, target);
    e1.set_operation(duel, activate);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_DRAW);
    e2.set_type(duel, EFFECT_TYPE_IGNITION);
    e2.set_range(duel, LOCATION_GRAVE);
    e2.set_count_limit(duel, 1, ID);
    e2.set_cost(duel, Cost::self_banish);
    e2.set_target(duel, drtg);
    e2.set_operation(duel, drop);
    c.register_effect(duel, e2);
}

fn filter(c: CardId, duel: &Duel, deck_count: usize) -> bool {
    !c.is_code(duel, ID)
        && (c.is_code(duel, CARD_DARK_MAGICIAN) || c.lists_code(duel, CARD_DARK_MAGICIAN) || c.lists_code(duel, CARD_DARK_MAGICIAN_GIRL))
        && if c.is_location(duel, LOCATION_DECK) {
            deck_count > 1
        } else {
            c.is_able_to_deck(duel)
        }
}

fn target(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        let deck_count = duel.get_field_group_count(tp, LOCATION_DECK, 0);
        return Ok(duel.is_existing_matching_card(
            |c, duel| filter(c, duel, deck_count),
            tp,
            LOCATION_HAND | LOCATION_DECK | LOCATION_GRAVE,
            0,
            1,
            None,
        ));
    }
    duel.set_operation_info(0, CATEGORY_TODECK, None, 1, tp, (LOCATION_HAND | LOCATION_GRAVE) as i32);
    Ok(false)
}

fn activate(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    let deck_count = duel.get_field_group_count(tp, LOCATION_DECK, 0);
    duel.hint(HINT_SELECTMSG, tp, aux::stringid(ID, 2));
    let filter = aux::necro_valley_filter(move |c, duel| filter(c, duel, deck_count));
    let g = duel.select_matching_card(tp, filter, tp, LOCATION_HAND | LOCATION_DECK | LOCATION_GRAVE, 0, 1, 1, None)?;
    let Some(tc) = g.get_first() else { return Ok(()) };
    if tc.is_location(duel, LOCATION_DECK) {
        duel.shuffle_deck(tp)?;
        duel.move_to_deck_top(tc.into())?;
    } else {
        duel.hint_selection(g);
        duel.send_to_deck(tc, PLAYER_NONE, SEQ_DECKTOP, REASON_EFFECT)?;
    }
    if !tc.is_location(duel, LOCATION_EXTRA) {
        duel.confirm_decktop(tp, 1)?;
    }
    Ok(())
}

fn drfilter(c: CardId, duel: &Duel) -> bool {
    (c.is_faceup(duel) || c.is_location(duel, LOCATION_GRAVE))
        && ((c.is_set_card(duel, &[SET_PALLADIUM]) && c.is_monster(duel))
            || c.is_code(duel, CARD_DARK_MAGICIAN)
            || c.is_code(duel, CARD_DARK_MAGICIAN_GIRL))
}

fn classes(duel: &Duel, tp: u8) -> usize {
    let locations = LOCATION_ONFIELD | LOCATION_GRAVE;
    duel.get_matching_group(drfilter, tp, locations, locations, None)
        .get_class_count(duel, |c, duel| i64::from(c.get_code(duel)))
}

fn drtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    let ct = classes(duel, tp);
    if chk == 0 {
        return Ok(ct > 0 && duel.is_player_can_draw(tp, ct));
    }
    duel.set_target_player(tp);
    duel.set_operation_info(0, CATEGORY_DRAW, None, 0, tp, ct as i32);
    Ok(false)
}

fn drop(duel: &mut Duel, a: &Args) -> Script<()> {
    let p: u8 = duel.get_chain_info(0, CHAININFO_TARGET_PLAYER);
    let ct = classes(duel, a.tp);
    duel.draw(p, ct as u32, REASON_EFFECT)?;
    Ok(())
}
