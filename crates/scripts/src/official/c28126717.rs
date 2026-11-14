// Floowandereeze and the Magnificent Map
use crate::prelude::*;
use crate::testing::{Check, DECK, HAND, MZONE};

const ID: u32 = 28126717;

pub const LISTED_SERIES: &[u16] = &[SET_FLOOWANDEREEZE];

// In the hand to activate, with Floowandereeze & Empen in the hand and Sangans on the field to tribute for it.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, HAND),
    (80611581, HAND),
    (80611581, HAND),
    (80611581, DECK),
    (26202165, MZONE),
    (26202165, MZONE),
];
// The reveal needs a Level 1 Floowandereeze monster, and none is ported yet.
pub const CHECKS: &[Check] = &[
    Check::Report(&Check::ResolvesEffect(aux::stringid(ID, 0))),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 0));
    e2.set_category(duel, CATEGORY_SUMMON + CATEGORY_REMOVE);
    e2.set_type(duel, EFFECT_TYPE_IGNITION);
    e2.set_range(duel, LOCATION_FZONE);
    e2.set_count_limit(duel, 1, ID);
    e2.set_target(duel, rmtg);
    e2.set_operation(duel, rmop);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 1));
    e3.set_category(duel, CATEGORY_SUMMON);
    e3.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e3.set_code(duel, EVENT_SUMMON_SUCCESS);
    e3.set_property(duel, EFFECT_FLAG_DELAY);
    e3.set_range(duel, LOCATION_FZONE);
    e3.set_count_limit(duel, 1, (ID, 1));
    e3.set_condition(duel, nscon);
    e3.set_target(duel, nstg);
    e3.set_operation(duel, nsop);
    c.register_effect(duel, e3);
}

fn filter(c: CardId, duel: &Duel, tp: u8) -> bool {
    let code = c.get_code(duel);
    c.is_set_card(duel, &[SET_FLOOWANDEREEZE])
        && c.is_level(duel, &[1])
        && c.is_summonable(duel, true, None)
        && !c.is_public(duel)
        && duel.is_existing_matching_card(|c, duel| filter2(c, duel, code), tp, LOCATION_DECK, 0, 1, None)
}

fn filter2(c: CardId, duel: &Duel, code: u32) -> bool {
    c.is_set_card(duel, &[SET_FLOOWANDEREEZE]) && c.is_able_to_remove(duel) && !c.is_code(duel, code)
}

fn rmtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(|c, duel| filter(c, duel, tp), tp, LOCATION_HAND, 0, 1, None));
    }
    duel.set_operation_info(0, CATEGORY_REMOVE, None, 1, tp, LOCATION_DECK as i32);
    Ok(false)
}

fn rmop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_CONFIRM);
    let g1 = duel.select_matching_card(tp, |c, duel| filter(c, duel, tp), tp, LOCATION_HAND, 0, 1, 1, None)?;
    let Some(sc) = g1.get_first() else { return Ok(()) };
    duel.confirm_cards(1 - tp, g1)?;
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_REMOVE);
    let code = sc.get_code(duel);
    let g2 = duel.select_matching_card(tp, |c, duel| filter2(c, duel, code), tp, LOCATION_DECK, 0, 1, 1, None)?;
    if !g2.is_empty() && duel.remove(g2, POS_FACEUP, REASON_EFFECT)? > 0 {
        duel.break_effect()?;
        duel.summon(tp, sc, true, None)?;
    }
    Ok(())
}

fn nscon(_: &Duel, a: &Args) -> bool {
    a.eg.is_some() && a.ep == 1 - a.tp
}

fn nsfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_FLOOWANDEREEZE]) && c.is_summonable(duel, true, None)
}

fn nstg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(nsfilter, a.tp, LOCATION_HAND | LOCATION_MZONE, 0, 1, None));
    }
    duel.set_operation_info(0, CATEGORY_SUMMON, None, 1, 0, 0);
    Ok(false)
}

fn nsop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SUMMON);
    let g = duel.select_matching_card(tp, nsfilter, tp, LOCATION_HAND | LOCATION_MZONE, 0, 1, 1, None)?;
    if let Some(sc) = g.get_first() {
        duel.summon(tp, sc, true, None)?;
    }
    Ok(())
}
