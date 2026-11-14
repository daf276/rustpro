// Kyoutou Waterfront
use engine::constants::MSG_ADD_COUNTER;

use crate::prelude::*;
use crate::testing::{Check, DECK, HAND, MZONE, OPPONENT};

pub const LISTED_SERIES: &[u16] = &[SET_KAIJU];
pub const COUNTER_PLACE_LIST: &[u16] = &[COUNTER_KAIJU];

// Kaiju monsters in the Deck to add.
pub const SUPPORT: &[(u32, u16)] = &[
    (63941210, DECK),
    (63941210, DECK),
    (29726552, DECK),
    (53129443, HAND),
    (12580477, HAND),
    (26202165, MZONE),
    (26202165, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(0), Check::Message(MSG_ADD_COUNTER)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_counter_permit(duel, COUNTER_KAIJU);
    c.set_counter_limit(duel, COUNTER_KAIJU, 5);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_CONTINUOUS);
    e2.set_code(duel, EVENT_TO_GRAVE);
    e2.set_range(duel, LOCATION_FZONE);
    e2.set_operation(duel, counter);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_category(duel, CATEGORY_TOHAND + CATEGORY_SEARCH);
    e3.set_type(duel, EFFECT_TYPE_IGNITION);
    e3.set_range(duel, LOCATION_FZONE);
    e3.set_count_limit(duel, 1, 0);
    e3.set_condition(duel, |duel, a| a.e.get_handler(duel).get_counter(duel, COUNTER_KAIJU) >= 3);
    e3.set_target(duel, thtg);
    e3.set_operation(duel, thop);
    c.register_effect(duel, e3);
    let e4 = duel.create_effect(c);
    e4.set_type(duel, EFFECT_TYPE_CONTINUOUS + EFFECT_TYPE_SINGLE);
    e4.set_code(duel, EFFECT_DESTROY_REPLACE);
    e4.set_property(duel, EFFECT_FLAG_SINGLE_RANGE);
    e4.set_range(duel, LOCATION_FZONE);
    e4.set_target(duel, desreptg);
    e4.set_operation(duel, desrepop);
    c.register_effect(duel, e4);
}

fn counter(duel: &mut Duel, a: &Args) -> Script<()> {
    let ct = a.eg.map_or(0, |eg| {
        eg.filter_count(duel, |c, duel| c.is_previous_location(duel, LOCATION_ONFIELD), None)
    });
    if ct > 0 {
        a.e.get_handler(duel).add_counter_full(duel, COUNTER_KAIJU, ct as u16, true);
    }
    Ok(())
}

fn thfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_KAIJU]) && c.is_monster(duel) && c.is_able_to_hand(duel)
}

fn thtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    stock::exists_target(duel, a, chk, thfilter, LOCATION_DECK, CATEGORY_TOHAND)
}

fn thop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    if !e.get_handler(duel).is_relate_to_effect(duel, e) {
        return Ok(());
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_ATOHAND);
    let g = duel.select_matching_card(tp, thfilter, tp, LOCATION_DECK, 0, 1, 1, None)?;
    if !g.is_empty() {
        duel.send_to_hand(g, PLAYER_NONE, REASON_EFFECT)?;
        duel.confirm_cards(1 - tp, g)?;
    }
    Ok(())
}

fn desreptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (c, tp) = (a.e.get_handler(duel), a.tp);
    if chk == 0 {
        return Ok(!c.is_reason(duel, REASON_REPLACE + REASON_RULE) && c.is_can_remove_counter(duel, tp, COUNTER_KAIJU, 1, REASON_EFFECT));
    }
    duel.select_effect_yes_no_full(tp, c, 96)
}

fn desrepop(duel: &mut Duel, a: &Args) -> Script<()> {
    a.e.get_handler(duel).remove_counter(duel, a.tp, COUNTER_KAIJU, 1, REASON_EFFECT)?;
    Ok(())
}
