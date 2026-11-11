// D/D Ghost
use crate::prelude::*;
use crate::testing::{Check, DECK, GRAVE, HAND, MZONE, REMOVED};

const ID: u32 = 33334269;

pub const LISTED_NAMES: &[u32] = &[ID];
// In the hand and on the field to be sent to the Graveyard by D/D Lamia's cost, with D/D Lance Soldier and Dark Contract
// with the Witch in the Graveyard and the Deck, and banished D/D Lance Soldier to return.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, MZONE),
    (19580308, HAND),
    (19580308, HAND),
    (67322708, GRAVE),
    (67322708, DECK),
    (67322708, REMOVED),
    (9765723, GRAVE),
    (9765723, DECK),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_TOGRAVE);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e1.set_code(duel, EVENT_TO_GRAVE);
    e1.set_property(duel, EFFECT_FLAG_DELAY + EFFECT_FLAG_CARD_TARGET);
    e1.set_target(duel, tgtg);
    e1.set_operation(duel, tgop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_TOGRAVE);
    e2.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e2.set_code(duel, EVENT_REMOVE);
    e2.set_property(duel, EFFECT_FLAG_DELAY + EFFECT_FLAG_CARD_TARGET);
    e2.set_target(duel, rtgtg);
    e2.set_operation(duel, rtgop);
    c.register_effect(duel, e2);
}

fn tgfilter1(c: CardId, duel: &Duel, tp: u8) -> bool {
    ((c.is_set_card(duel, &[SET_DD]) && c.is_monster(duel) && !c.is_code(duel, ID)) || c.is_set_card(duel, &[SET_DARK_CONTRACT]))
        && duel.is_existing_matching_card(|sc, duel| tgfilter2(sc, duel, c.get_code(duel)), tp, LOCATION_DECK, 0, 1, None)
}

fn tgfilter2(c: CardId, duel: &Duel, cd: u32) -> bool {
    c.is_code(duel, cd) && c.is_able_to_grave(duel)
}

fn tgtg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_GRAVE) && chkc.is_controler(duel, tp) && chkc.is_code(duel, e.get_label(duel) as u32));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(|c, duel| tgfilter1(c, duel, tp), tp, LOCATION_GRAVE, 0, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TARGET);
    let g = duel.select_target(tp, |c, duel| tgfilter1(c, duel, tp), tp, LOCATION_GRAVE, 0, 1, 1, None)?;
    if let Some(tc) = g.and_then(|g| g.get_first()) {
        e.set_label(duel, tc.get_code(duel).into());
    }
    duel.set_operation_info(0, CATEGORY_TOGRAVE, None, 1, tp, LOCATION_DECK as i32);
    Ok(false)
}

fn tgop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    let Some(tc) = duel.get_first_target() else { return Ok(()) };
    if tc.is_relate_to_effect(duel, a.e) {
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_TOGRAVE);
        let cd = tc.get_code(duel);
        let g = duel.select_matching_card(tp, |c, duel| tgfilter2(c, duel, cd), tp, LOCATION_DECK, 0, 1, 1, None)?;
        if !g.is_empty() {
            duel.send_to_grave(g, REASON_EFFECT)?;
        }
    }
    Ok(())
}

fn rtgfilter(c: CardId, duel: &Duel) -> bool {
    c.is_faceup(duel)
        && ((c.is_set_card(duel, &[SET_DD]) && c.is_monster(duel) && !c.is_code(duel, ID)) || c.is_set_card(duel, &[SET_DARK_CONTRACT]))
}

fn rtgtg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_REMOVED) && chkc.is_controler(duel, tp) && rtgfilter(chkc, duel));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(rtgfilter, tp, LOCATION_REMOVED, 0, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TARGET);
    let g = duel.select_target(tp, rtgfilter, tp, LOCATION_REMOVED, 0, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_TOGRAVE, g, 1, 0, 0);
    Ok(false)
}

fn rtgop(duel: &mut Duel, a: &Args) -> Script<()> {
    if let Some(tc) = duel.get_first_target()
        && tc.is_relate_to_effect(duel, a.e)
    {
        duel.send_to_grave(tc, REASON_EFFECT | REASON_RETURN)?;
    }
    Ok(())
}
