// Ext Ryzeal
use super::c8633261::extra_deck_lock;
use crate::prelude::*;
use crate::testing::{Check, DECK, EXTRA, HAND};

const ID: u32 = 34022970;

pub const LISTED_NAMES: &[u32] = &[ID];

// In the hand, Special Summoned by sending an Xyz Monster from the Extra Deck to the Graveyard, with FIRE Thunder Ryzeal
// monsters in the Deck to search.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, HAND),
    (34909328, EXTRA),
    (7511613, EXTRA),
    (35844557, DECK),
    (72238166, DECK),
];
pub const CHECKS: &[Check] = &[Check::SpecialSummoned, Check::ResolvesEffect(aux::stringid(ID, 1))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_code(duel, EFFECT_SPSUMMON_PROC);
    e1.set_property(duel, EFFECT_FLAG_UNCOPYABLE);
    e1.set_range(duel, LOCATION_HAND);
    e1.set_count_limit_full(duel, 1, ID, EFFECT_COUNT_CODE_OATH);
    e1.set_condition(duel, selfspcon);
    e1.set_target(duel, selfsptg);
    e1.set_operation(duel, selfspop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_TOHAND + CATEGORY_SEARCH);
    e2.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e2.set_property(duel, EFFECT_FLAG_DELAY);
    e2.set_code(duel, EVENT_SUMMON_SUCCESS);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_target(duel, thtg);
    e2.set_operation(duel, thop);
    c.register_effect(duel, e2);
    let e3 = e2.clone_effect(duel);
    e3.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    c.register_effect(duel, e3);
}

fn selfspconfilter(c: CardId, duel: &Duel) -> bool {
    c.is_type(duel, TYPE_XYZ) && c.is_able_to_grave_as_cost(duel)
}

fn selfspcon(duel: &Duel, a: &Args) -> bool {
    if a.c.is_none() {
        return true;
    }
    let tp = a.e.get_handler_player(duel);
    duel.is_existing_matching_card(selfspconfilter, tp, LOCATION_EXTRA, 0, 1, None) && duel.get_location_count(tp, LOCATION_MZONE) > 0
}

fn selfsptg(duel: &mut Duel, a: &Args, _: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    let rg = duel.get_matching_group(selfspconfilter, tp, LOCATION_EXTRA, 0, None);
    let g = aux::select_unselect_group_full(duel, rg, 1, 1, &aux::chkf_mmz(tp, 1), tp, HINTMSG_TOGRAVE, None, true)?;
    if g.is_empty() {
        return Ok(false);
    }
    a.e.set_label_object_group(duel, Some(g));
    Ok(true)
}

fn selfspop(duel: &mut Duel, a: &Args) -> Script<()> {
    let Some(g) = a.e.get_label_object_group(duel) else { return Ok(()) };
    duel.send_to_grave(g, REASON_COST)?;
    if let Some(c) = a.c {
        extra_deck_lock(duel, c, a.tp, aux::stringid(ID, 2));
    }
    Ok(())
}

fn thfilter(c: CardId, duel: &Duel) -> bool {
    c.is_attribute(duel, ATTRIBUTE_FIRE) && c.is_race(duel, RACE_THUNDER) && c.is_able_to_hand(duel)
}

fn thconfilter(c: CardId, duel: &Duel) -> bool {
    c.is_faceup(duel) && !(c.is_level(duel, &[4]) || c.is_rank(duel, &[4]))
}

fn thtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(thfilter, tp, LOCATION_DECK, 0, 1, None)
            && !duel.is_existing_matching_card(thconfilter, tp, LOCATION_MZONE, 0, 1, None));
    }
    duel.set_operation_info(0, CATEGORY_TOHAND, None, 1, tp, LOCATION_DECK as i32);
    Ok(false)
}

fn thop(duel: &mut Duel, a: &Args) -> Script<()> {
    stock::search(duel, a, thfilter, LOCATION_DECK)
}
