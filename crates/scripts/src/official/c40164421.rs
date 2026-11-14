// Minerva, Lightsworn Maiden
use crate::prelude::*;
use crate::testing::{Check, DECK, GRAVE, HAND, MZONE};

const ID: u32 = 40164421;

pub const LISTED_SERIES: &[u16] = &[SET_LIGHTSWORN];
// In the hand and on the field with Lightsworn monsters in the Graveyard and Dragons (Galaxy Serpent, Alexandrite
// Dragon) in the Deck to search, and Raiden to send her from the Deck to the Graveyard.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, DECK),
    (ID, MZONE),
    (77558536, MZONE),
    (22624373, GRAVE),
    (58996430, GRAVE),
    (11066358, DECK),
    (11066358, DECK),
    (43096270, DECK),
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
    e1.set_code(duel, EVENT_SUMMON_SUCCESS);
    e1.set_target(duel, thtg);
    e1.set_operation(duel, thop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_DECKDES);
    e2.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_F);
    e2.set_code(duel, EVENT_TO_GRAVE);
    e2.set_condition(duel, |duel, a| {
        a.e.get_handler(duel).is_previous_location(duel, LOCATION_DECK | LOCATION_HAND)
    });
    e2.set_target(duel, distg);
    e2.set_operation(duel, disop);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 2));
    e3.set_category(duel, CATEGORY_DECKDES);
    e3.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_F);
    e3.set_code(duel, EVENT_PHASE + PHASE_END);
    e3.set_range(duel, LOCATION_MZONE);
    e3.set_count_limit(duel, 1, 0);
    e3.set_condition(duel, |duel, a| a.tp == duel.get_turn_player());
    e3.set_target(duel, distg2);
    e3.set_operation(duel, |duel, a| {
        duel.discard_deck(a.tp, 2, REASON_EFFECT)?;
        Ok(())
    });
    c.register_effect(duel, e3);
}

fn cfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_LIGHTSWORN]) && c.is_monster(duel)
}

fn thfilter(c: CardId, duel: &Duel, lv: usize) -> bool {
    c.is_level_below(duel, lv as u32) && c.is_race(duel, RACE_DRAGON) && c.is_attribute(duel, ATTRIBUTE_LIGHT) && c.is_able_to_hand(duel)
}

fn graveclasses(duel: &Duel, tp: u8) -> usize {
    duel.get_matching_group(cfilter, tp, LOCATION_GRAVE, 0, None)
        .get_class_count(duel, |c, duel| i64::from(c.get_code(duel)))
}

fn thtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        let ct = graveclasses(duel, tp);
        return Ok(duel.is_existing_matching_card(|c, duel| thfilter(c, duel, ct), tp, LOCATION_DECK, 0, 1, None));
    }
    duel.set_operation_info(0, CATEGORY_TOHAND, None, 1, tp, LOCATION_DECK as i32);
    Ok(false)
}

fn thop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    let ct = graveclasses(duel, tp);
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_ATOHAND);
    let sg = duel.select_matching_card(tp, |c, duel| thfilter(c, duel, ct), tp, LOCATION_DECK, 0, 1, 1, None)?;
    if !sg.is_empty() {
        duel.send_to_hand(sg, PLAYER_NONE, REASON_EFFECT)?;
        duel.confirm_cards(1 - tp, sg)?;
    }
    Ok(())
}

fn distg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(true);
    }
    duel.set_target_player(tp);
    duel.set_target_param(1);
    duel.set_operation_info(0, CATEGORY_DECKDES, None, 0, tp, 1);
    Ok(false)
}

fn disop(duel: &mut Duel, _: &Args) -> Script<()> {
    let p: u8 = duel.get_chain_info(0, CHAININFO_TARGET_PLAYER);
    let d: u32 = duel.get_chain_info(0, CHAININFO_TARGET_PARAM);
    duel.discard_deck(p, d as u16, REASON_EFFECT)?;
    Ok(())
}

fn distg2(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(true);
    }
    duel.set_operation_info(0, CATEGORY_DECKDES, None, 0, a.tp, 2);
    Ok(false)
}
