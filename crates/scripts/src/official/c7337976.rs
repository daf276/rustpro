// Void Apocalypse
use crate::prelude::*;
use crate::testing::{Check, DECK, MZONE, OPPONENT, SZONE};

use super::c78748366::thcost as tgcost;

const ID: u32 = 7337976;

pub const LISTED_SERIES: &[u16] = &[SET_INFERNOID];
// Face-up in the Spell & Trap Zone with Infernoids in the Deck, against the opponent's Despian Quaeritis.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, SZONE),
    (ID, SZONE),
    (60551528, DECK),
    (58446973, DECK),
    (26034577, DECK),
    (72272462, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 0));
    e2.set_category(duel, CATEGORY_TOGRAVE);
    e2.set_type(duel, EFFECT_TYPE_IGNITION);
    e2.set_range(duel, LOCATION_SZONE);
    e2.set_count_limit(duel, 1, ID);
    e2.set_cost(duel, tgcost);
    e2.set_target(duel, tgtg);
    e2.set_operation(duel, tgop);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 1));
    e3.set_category(duel, CATEGORY_TOGRAVE);
    e3.set_type(duel, EFFECT_TYPE_IGNITION);
    e3.set_range(duel, LOCATION_SZONE);
    e3.set_count_limit(duel, 1, ID);
    e3.set_cost(duel, Cost::self_to_grave);
    e3.set_target(duel, tgtg2);
    e3.set_operation(duel, tgop2);
    c.register_effect(duel, e3);
}

fn tgfilter(c: CardId, duel: &Duel) -> bool {
    c.is_race(duel, RACE_FIEND) && c.is_able_to_grave(duel)
}

fn tgtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    stock::exists_target(duel, a, chk, tgfilter, LOCATION_DECK, CATEGORY_TOGRAVE)
}

fn tgop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    if !a.e.get_handler(duel).is_relate_to_effect(duel, a.e) {
        return Ok(());
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TOGRAVE);
    let g = duel.select_matching_card(tp, tgfilter, tp, LOCATION_DECK, 0, 1, 1, None)?;
    if !g.is_empty() {
        duel.send_to_grave(g, REASON_EFFECT)?;
    }
    Ok(())
}

fn tgfilter2(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_INFERNOID]) && c.is_monster(duel) && c.is_able_to_grave(duel)
}

fn from_extra(c: CardId, duel: &Duel) -> bool {
    c.is_summon_location(duel, LOCATION_EXTRA)
}

fn tgtg2(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(
            duel.is_existing_matching_card(tgfilter2, tp, LOCATION_HAND | LOCATION_DECK, 0, 1, None)
                && duel.is_existing_matching_card(from_extra, tp, 0, LOCATION_MZONE, 1, None),
        );
    }
    duel.set_operation_info(0, CATEGORY_TOGRAVE, None, 1, tp, (LOCATION_HAND | LOCATION_DECK) as i32);
    Ok(false)
}

fn tgop2(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    let ct = duel.get_matching_group_count(from_extra, tp, 0, LOCATION_MZONE, None);
    if ct < 1 {
        return Ok(());
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TOGRAVE);
    let g = duel.select_matching_card(tp, tgfilter2, tp, LOCATION_HAND | LOCATION_DECK, 0, 1, ct as u8, None)?;
    if !g.is_empty() {
        duel.send_to_grave(g, REASON_EFFECT)?;
    }
    Ok(())
}
