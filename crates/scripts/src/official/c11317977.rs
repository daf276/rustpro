// Lunalight Black Sheep
use crate::prelude::*;
use crate::testing::{Check, DECK, EXTRA, HAND};

const ID: u32 = 11317977;
const LUNALIGHT_SILVER_HOUND: u32 = 35763582;
const LUNALIGHT_PERFUME_DANCER: u32 = 81196066;

pub const LISTED_NAMES: &[u32] = &[CARD_POLYMERIZATION, ID];

// In hand with Polymerization in the Deck to search, and Polymerization and Lunalight Silver Hound in hand to Fusion Summon
// Lunalight Perfume Dancer from the Extra Deck with it.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (CARD_POLYMERIZATION, DECK),
    (CARD_POLYMERIZATION, HAND),
    (LUNALIGHT_SILVER_HOUND, HAND),
    (LUNALIGHT_PERFUME_DANCER, EXTRA),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_range(duel, LOCATION_HAND);
    e1.set_cost(duel, Cost::self_discard);
    e1.set_target(duel, efftg);
    e1.set_operation(duel, effop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_TOHAND);
    e2.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e2.set_property(duel, EFFECT_FLAG_DELAY);
    e2.set_code(duel, EVENT_BE_MATERIAL);
    e2.set_condition(duel, |duel, a| {
        a.e.get_handler(duel).is_location(duel, LOCATION_GRAVE) && a.r == REASON_FUSION
    });
    e2.set_target(duel, exgythtg);
    e2.set_operation(duel, exgythop);
    c.register_effect(duel, e2);
}

fn gythfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_LUNALIGHT]) && c.is_monster(duel) && !c.is_code(duel, ID) && c.is_able_to_hand(duel)
}

fn polythfilter(c: CardId, duel: &Duel) -> bool {
    c.is_code(duel, CARD_POLYMERIZATION) && c.is_able_to_hand(duel)
}

fn efftg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let b1 = duel.is_existing_matching_card(gythfilter, tp, LOCATION_GRAVE, 0, 1, None);
    let b2 = duel.is_existing_matching_card(polythfilter, tp, LOCATION_DECK, 0, 1, None);
    if chk == 0 {
        return Ok(b1 || b2);
    }
    let op = duel.select_effect(tp, &[(b1, aux::stringid(ID, 2)), (b2, aux::stringid(ID, 3))])?;
    e.set_label(duel, op.map_or(0, |op| op as i64));
    if op == Some(1) {
        e.set_category(duel, CATEGORY_TOHAND);
        duel.set_operation_info(0, CATEGORY_TOHAND, None, 1, tp, LOCATION_GRAVE as i32);
    } else if op == Some(2) {
        e.set_category(duel, CATEGORY_TOHAND + CATEGORY_SEARCH);
        duel.set_operation_info(0, CATEGORY_TOHAND, None, 1, tp, LOCATION_DECK as i32);
    }
    Ok(false)
}

fn effop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    let (location, filter): (u32, fn(CardId, &Duel) -> bool) = match a.e.get_label(duel) {
        1 => (LOCATION_GRAVE, gythfilter),
        2 => (LOCATION_DECK, polythfilter),
        _ => return Ok(()),
    };
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_ATOHAND);
    let g = duel.select_matching_card(tp, filter, tp, location, 0, 1, 1, None)?;
    if !g.is_empty() {
        duel.send_to_hand(g, PLAYER_NONE, REASON_EFFECT)?;
        duel.confirm_cards(1 - tp, g)?;
    }
    Ok(())
}

fn exgythfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_LUNALIGHT])
        && c.is_monster(duel)
        && !c.is_code(duel, ID)
        && c.is_able_to_hand(duel)
        && ((c.is_location(duel, LOCATION_EXTRA) && c.is_faceup(duel) && c.is_type(duel, TYPE_PENDULUM))
            || c.is_location(duel, LOCATION_GRAVE))
}

fn exgythtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(exgythfilter, tp, LOCATION_EXTRA | LOCATION_GRAVE, 0, 1, None));
    }
    duel.set_operation_info(0, CATEGORY_TOHAND, None, 1, tp, (LOCATION_EXTRA | LOCATION_GRAVE) as i32);
    Ok(false)
}

fn exgythop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_ATOHAND);
    let filter = aux::necro_valley_filter(exgythfilter);
    let g = duel.select_matching_card(tp, filter, tp, LOCATION_EXTRA | LOCATION_GRAVE, 0, 1, 1, None)?;
    if !g.is_empty() {
        duel.send_to_hand(g, PLAYER_NONE, REASON_EFFECT)?;
        duel.confirm_cards(1 - tp, g)?;
    }
    Ok(())
}
