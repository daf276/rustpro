// Blue-Eyes Abyss Dragon
use crate::prelude::*;
use crate::testing::{Check, DECK, GRAVE, HAND, MZONE, OPPONENT};

const ID: u32 = 64202399;

pub const LISTED_NAMES: &[u32] = &[CARD_BLUEEYES_W_DRAGON, CARD_POLYMERIZATION];

// In the hand and Graveyard with Blue-Eyes White Dragon in the Graveyard, Polymerization and a Level 8 Dragon in the Deck to search,
// and Blue-Eyes Ultimate Spirit Dragon on the field, which Raigeki destroys to Special Summon a LIGHT Dragon from the Graveyard, and Monster Reborn.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, GRAVE),
    (CARD_BLUEEYES_W_DRAGON, GRAVE),
    (CARD_BLUEEYES_W_DRAGON, DECK),
    (CARD_POLYMERIZATION, DECK),
    (CARD_POLYMERIZATION, DECK),
    (89604813, MZONE),
    (12580477, HAND | OPPONENT),
    (83764718, HAND),
    (83764718, DECK),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
    Check::ResolvesEffect(aux::stringid(ID, 2)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_TOHAND + CATEGORY_SEARCH);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e1.set_property(duel, EFFECT_FLAG_DELAY);
    e1.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e1.set_condition(duel, condition);
    e1.set_count_limit(duel, 1, ID);
    e1.set_target(duel, thtg1);
    e1.set_operation(duel, thop1);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_TOHAND + CATEGORY_SEARCH);
    e2.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_code(duel, EVENT_PHASE + PHASE_END);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_condition(duel, |duel, a| duel.is_turn_player(a.tp) && condition(duel, a));
    e2.set_target(duel, thtg2);
    e2.set_operation(duel, thop2);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 2));
    e3.set_category(duel, CATEGORY_ATKCHANGE);
    e3.set_type(duel, EFFECT_TYPE_IGNITION);
    e3.set_range(duel, LOCATION_GRAVE);
    e3.set_count_limit(duel, 1, (ID, 2));
    e3.set_condition(duel, condition);
    e3.set_cost(duel, Cost::self_banish);
    e3.set_target(duel, target);
    e3.set_operation(duel, operation);
    c.register_effect(duel, e3);
}

fn cfilter(c: CardId, duel: &Duel) -> bool {
    c.is_code(duel, CARD_BLUEEYES_W_DRAGON) && (c.is_location(duel, LOCATION_GRAVE) || c.is_on_field(duel) && c.is_faceup(duel))
}

fn condition(duel: &Duel, a: &Args) -> bool {
    duel.is_existing_matching_card(cfilter, a.tp, LOCATION_ONFIELD | LOCATION_GRAVE, 0, 1, None)
}

fn thfilter1(c: CardId, duel: &Duel) -> bool {
    (c.is_code(duel, CARD_POLYMERIZATION) || c.is_ritual_spell(duel)) && c.is_able_to_hand(duel)
}

fn thtg1(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    stock::exists_target(duel, a, chk, thfilter1, LOCATION_DECK, CATEGORY_TOHAND)
}

fn search(duel: &mut Duel, tp: u8, filter: fn(CardId, &Duel) -> bool) -> Script<()> {
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_ATOHAND);
    let g = duel.select_matching_card(tp, filter, tp, LOCATION_DECK, 0, 1, 1, None)?;
    if !g.is_empty() {
        duel.send_to_hand(g, PLAYER_NONE, REASON_EFFECT)?;
        duel.confirm_cards(1 - tp, g)?;
    }
    Ok(())
}

fn thop1(duel: &mut Duel, a: &Args) -> Script<()> {
    search(duel, a.tp, thfilter1)
}

fn thfilter2(c: CardId, duel: &Duel) -> bool {
    c.is_level_above(duel, 8) && c.is_race(duel, RACE_DRAGON) && c.is_able_to_hand(duel)
}

fn thtg2(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    stock::exists_target(duel, a, chk, thfilter2, LOCATION_DECK, CATEGORY_TOHAND)
}

fn thop2(duel: &mut Duel, a: &Args) -> Script<()> {
    search(duel, a.tp, thfilter2)
}

fn filter(c: CardId, duel: &Duel) -> bool {
    c.is_faceup(duel) && c.is_level_above(duel, 8) && c.is_race(duel, RACE_DRAGON)
}

fn target(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    Ok(chk == 0 && duel.is_existing_matching_card(filter, a.tp, LOCATION_MZONE, 0, 1, None))
}

fn operation(duel: &mut Duel, a: &Args) -> Script<()> {
    let c = a.e.get_handler(duel);
    for tc in duel.get_matching_group(filter, a.tp, LOCATION_MZONE, 0, None) {
        let e1 = duel.create_effect(c);
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_code(duel, EFFECT_UPDATE_ATTACK);
        e1.set_value(duel, 1000);
        e1.set_reset(duel, RESET_EVENT | RESETS_STANDARD);
        tc.register_effect(duel, e1);
    }
    Ok(())
}
