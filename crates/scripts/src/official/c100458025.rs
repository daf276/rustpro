// "Raise Moon" the City that Never Sleeps
use super::c100458024::drconfilter;
use crate::prelude::*;
use crate::testing::{Check, GRAVE, MZONE};

const ID: u32 = 100458025;
const CHOICE: u8 = 0;

pub const LISTED_SERIES: &[u16] = &[SET_RAISE_MOON];

// Placed in the Field Zone, Raise Moon cards in the Graveyard to shuffle in, and a Spellcaster Raise Moon Xyz Monster.
pub const SUPPORT: &[(u32, u16)] = &[(ID, 0), (100458017, GRAVE), (100458024, GRAVE), (100458022, MZONE)];
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
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_code(duel, EFFECT_INDESTRUCTABLE_COUNT);
    e1.set_range(duel, LOCATION_SZONE);
    e1.set_target_range(duel, LOCATION_MZONE, 0);
    e1.set_target_card(duel, |duel, _, c| c.is_level(duel, &[7]) && c.is_set_card(duel, &[SET_RAISE_MOON]));
    e1.set_value(duel, Value::Reason(|_, _, _, r, _| r & REASON_EFFECT == REASON_EFFECT));
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 0));
    e2.set_category(duel, CATEGORY_TODECK);
    e2.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_F);
    e2.set_code(duel, EVENT_PHASE + PHASE_END);
    e2.set_range(duel, LOCATION_SZONE);
    e2.set_count_limit(duel, 1, 0);
    e2.set_target(duel, tdtg);
    e2.set_operation(duel, tdop);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 1));
    e3.set_type(duel, EFFECT_TYPE_IGNITION);
    e3.set_range(duel, LOCATION_SZONE);
    e3.set_count_limit(duel, 1, 0);
    e3.set_target(duel, efftg);
    e3.set_operation(duel, effop);
    c.register_effect(duel, e3);
}

fn tdfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_RAISE_MOON]) && c.is_able_to_deck(duel) && c.is_faceup(duel)
}

fn tdtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(true);
    }
    let g = duel.get_matching_group(tdfilter, a.tp, LOCATION_GRAVE | LOCATION_REMOVED, 0, None);
    duel.set_operation_info(0, CATEGORY_TODECK, Some(g), g.len() as u8, a.tp, 0);
    Ok(false)
}

fn tdop(duel: &mut Duel, a: &Args) -> Script<()> {
    let g = duel.get_matching_group(aux::necro_valley_filter(tdfilter), a.tp, LOCATION_GRAVE | LOCATION_REMOVED, 0, None);
    if !g.is_empty() {
        duel.send_to_deck(g, PLAYER_NONE, SEQ_DECKSHUFFLE, REASON_EFFECT)?;
    }
    Ok(())
}

fn efftg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let option_1 =
        duel.is_existing_matching_card(|c, duel| c.is_able_to_deck(duel), tp, LOCATION_HAND, 0, 1, None) && duel.is_player_can_draw(tp, 0);
    let option_2 = duel.is_existing_matching_card(drconfilter, tp, LOCATION_MZONE, 0, 1, None) && duel.is_player_can_draw(tp, 1);
    if chk == 0 {
        return Ok(option_1 || option_2);
    }
    let choice = duel.select_effect(tp, &[(option_1, aux::stringid(ID, 2)), (option_2, aux::stringid(ID, 3))])?;
    e.set_chain_data(duel, CHOICE, choice.map_or(0, |choice| choice as i64));
    if choice == Some(1) {
        e.set_category(duel, CATEGORY_TODECK + CATEGORY_DRAW);
        duel.set_operation_info(0, CATEGORY_TODECK, None, 1, tp, 1);
        duel.set_operation_info(0, CATEGORY_DRAW, None, 0, tp, 1);
    } else if choice == Some(2) {
        e.set_category(duel, CATEGORY_DRAW);
        duel.set_operation_info(0, CATEGORY_DRAW, None, 0, tp, 1);
    }
    Ok(false)
}

fn effop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    let choice = a.e.get_chain_data(duel, CHOICE).unwrap_or(0);
    if choice == 1 {
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_TODECK);
        let g = duel.select_matching_card(tp, |c, duel| c.is_able_to_deck(duel), tp, LOCATION_HAND, 0, 1, 1, None)?;
        let Some(sc) = g.get_first() else { return Ok(()) };
        let mut seq_op = 0;
        if duel.get_field_group_count(tp, LOCATION_DECK, 0) > 0 {
            seq_op = duel.select_option(tp, &[aux::stringid(ID, 4), aux::stringid(ID, 5)])?;
        }
        if duel.send_to_deck(sc, PLAYER_NONE, seq_op, REASON_EFFECT)? > 0
            && sc.is_location(duel, LOCATION_DECK)
            && duel.is_player_can_draw(tp, 0)
        {
            duel.break_effect()?;
            duel.draw(tp, 1, REASON_EFFECT)?;
        }
    } else if choice == 2 {
        duel.draw(tp, 1, REASON_EFFECT)?;
    }
    Ok(())
}
