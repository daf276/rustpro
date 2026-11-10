// Void Liberation
use crate::prelude::*;
use crate::testing::{Check, DECK, GRAVE, HAND};

const ID: u32 = 61345801;

// Shaddoll Squamata (a monster) in the hand to reveal, with Shaddoll Fusion (a Spell) and Shaddoll Core (a Trap) in the Deck to add.
// No Infernoid monster is ported, so the Graveyard effect is only reported.
pub const SUPPORT: &[(u32, u16)] = &[
    (30328508, HAND),
    (44394295, DECK),
    (44394295, DECK),
    (44394295, DECK),
    (44394295, DECK),
    (4904633, DECK),
    (4904633, DECK),
    (4904633, DECK),
    (4904633, DECK),
    (ID, GRAVE),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::Report(&Check::ResolvesEffect(aux::stringid(ID, 1))),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_TOHAND + CATEGORY_SEARCH + CATEGORY_HANDES);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_count_limit(duel, 1, (ID, 0));
    e1.set_cost(duel, revealcost);
    e1.set_target(duel, shaddollthtg);
    e1.set_operation(duel, shaddollthop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_TOHAND + CATEGORY_SEARCH);
    e2.set_type(duel, EFFECT_TYPE_IGNITION);
    e2.set_range(duel, LOCATION_GRAVE);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_cost(duel, Cost::self_banish);
    e2.set_target(duel, infernoidthtg);
    e2.set_operation(duel, infernoidthop);
    c.register_effect(duel, e2);
}

fn shaddollthfilter(c: CardId, duel: &Duel, main_type: u32) -> bool {
    c.is_set_card(duel, &[SET_SHADDOLL]) && c.is_able_to_hand(duel) && !c.is_type(duel, main_type)
}

fn revealfilter(c: CardId, duel: &Duel, _: EffectId, tp: u8) -> bool {
    let main_type = c.get_main_card_type(duel);
    c.is_set_card(duel, &[SET_SHADDOLL])
        && !c.is_public(duel)
        && duel
            .get_matching_group(|c, duel| shaddollthfilter(c, duel, main_type), tp, LOCATION_DECK, 0, None)
            .get_class_count(duel, |c, duel| i64::from(c.get_main_card_type(duel)))
            == 2
}

fn revealcost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let record: fn(&mut Duel, &Args, CardSet) = |duel, a, g| {
        if let Some(c) = g.get_first() {
            let main_type = c.get_main_card_type(duel);
            a.e.set_label(duel, i64::from(main_type));
        }
    };
    Cost::reveal(duel, a, chk, revealfilter, true, (1, 1), Some(record), LOCATION_HAND)
}

fn shaddollthtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(true);
    }
    duel.set_operation_info(0, CATEGORY_TOHAND, None, 2, a.tp, LOCATION_DECK as i32);
    duel.set_operation_info(0, CATEGORY_HANDES, None, 0, a.tp, 1);
    Ok(false)
}

fn shaddollthop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let main_type = e.get_label(duel) as u32;
    let g = duel.get_matching_group(|c, duel| shaddollthfilter(c, duel, main_type), tp, LOCATION_DECK, 0, None);
    if g.len() < 2 {
        return Ok(());
    }
    let sg = aux::select_unselect_group(
        duel,
        g,
        2,
        2,
        &aux::dpcheck(|c, duel| i64::from(c.get_main_card_type(duel))),
        tp,
        HINTMSG_ATOHAND,
    )?;
    if sg.len() == 2 && duel.send_to_hand(sg, PLAYER_NONE, REASON_EFFECT)? > 0 {
        duel.confirm_cards(1 - tp, sg)?;
        duel.shuffle_hand(tp)?;
        duel.break_effect()?;
        duel.discard_hand(tp, aux::TRUE, 1, 1, REASON_EFFECT | REASON_DISCARD)?;
    }
    Ok(())
}

fn infernoidthfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_INFERNOID]) && c.is_monster(duel) && c.is_able_to_hand(duel)
}

fn infernoidthtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        let handler = a.e.get_handler(duel);
        return Ok(duel.is_existing_matching_card(infernoidthfilter, tp, LOCATION_DECK | LOCATION_GRAVE, 0, 1, Some(handler.into())));
    }
    duel.set_operation_info(0, CATEGORY_TOHAND, None, 1, tp, (LOCATION_DECK | LOCATION_GRAVE) as i32);
    Ok(false)
}

fn infernoidthop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_ATOHAND);
    let g = duel.select_matching_card(
        tp,
        aux::necro_valley_filter(infernoidthfilter),
        tp,
        LOCATION_DECK | LOCATION_GRAVE,
        0,
        1,
        1,
        None,
    )?;
    if !g.is_empty() {
        duel.send_to_hand(g, PLAYER_NONE, REASON_EFFECT)?;
        duel.confirm_cards(1 - tp, g)?;
    }
    Ok(())
}
