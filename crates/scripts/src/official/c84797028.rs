// Clockwork Night
use crate::prelude::*;
use crate::testing::{Check, DECK, GRAVE, HAND};

const ID: u32 = 84797028;

// In the hand and the Graveyard, with Overdrive, an EARTH Machine, in the Deck to search.
pub const SUPPORT: &[(u32, u16)] = &[(ID, HAND), (ID, GRAVE), (2311603, DECK), (2311603, DECK)];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_count_limit_full(duel, 1, ID, EFFECT_COUNT_CODE_OATH);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_FIELD);
    e2.set_code(duel, EFFECT_CHANGE_RACE);
    e2.set_range(duel, LOCATION_SZONE);
    e2.set_target_range(duel, LOCATION_MZONE, LOCATION_MZONE);
    e2.set_value(duel, RACE_MACHINE as i32);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_type(duel, EFFECT_TYPE_FIELD);
    e3.set_code(duel, EFFECT_UPDATE_ATTACK);
    e3.set_range(duel, LOCATION_SZONE);
    e3.set_target_range(duel, LOCATION_MZONE, LOCATION_MZONE);
    e3.set_target_card(duel, |duel, _, c| c.is_race(duel, RACE_MACHINE));
    e3.set_value(duel, Value::Card(val));
    c.register_effect(duel, e3);
    let e4 = e3.clone_effect(duel);
    e4.set_code(duel, EFFECT_UPDATE_DEFENSE);
    c.register_effect(duel, e4);
    let e5 = duel.create_effect(c);
    e5.set_description(duel, aux::stringid(ID, 0));
    e5.set_category(duel, CATEGORY_TOHAND + CATEGORY_SEARCH);
    e5.set_type(duel, EFFECT_TYPE_IGNITION);
    e5.set_range(duel, LOCATION_GRAVE);
    e5.set_count_limit(duel, 1, (ID, 1));
    e5.set_cost(duel, thcost);
    e5.set_target(duel, thtg);
    e5.set_operation(duel, thop);
    c.register_effect(duel, e5);
}

fn val(duel: &Duel, e: EffectId, c: CardId) -> i32 {
    if c.is_controler(duel, e.get_handler_player(duel)) {
        500
    } else {
        -500
    }
}

fn thcost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let (c, tp) = (a.e.get_handler(duel), a.tp);
    if chk == 0 {
        return Ok(
            c.is_able_to_remove_as_cost(duel) && duel.is_existing_matching_card(CardId::is_discardable, tp, LOCATION_HAND, 0, 1, None)
        );
    }
    duel.remove(c, POS_FACEUP, REASON_COST)?;
    duel.discard_hand(tp, CardId::is_discardable, 1, 1, REASON_COST | REASON_DISCARD)?;
    Ok(false)
}

fn thfilter(c: CardId, duel: &Duel) -> bool {
    c.is_attribute(duel, ATTRIBUTE_EARTH) && c.is_race(duel, RACE_MACHINE) && c.is_able_to_hand(duel)
}

fn thtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    stock::exists_target(duel, a, chk, thfilter, LOCATION_DECK, CATEGORY_TOHAND)
}

fn thop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_ATOHAND);
    let g = duel.select_matching_card(tp, thfilter, tp, LOCATION_DECK, 0, 1, 1, None)?;
    if g.is_empty() {
        return Ok(());
    }
    duel.send_to_hand(g, PLAYER_NONE, REASON_EFFECT)?;
    duel.confirm_cards(1 - tp, g)?;
    Ok(())
}
