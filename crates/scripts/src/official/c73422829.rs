// Super Quantum White Layer
use crate::prelude::*;
use crate::testing::{Check, DECK, GRAVE, HAND, MZONE};

const ID: u32 = 73422829;
const ALPHAN: u32 = 58753372;

pub const LISTED_NAMES: &[u32] = &[ALPHAN];

// Blue Layer (a non-LIGHT "Super Quant" monster) in the hand, field and Deck for the cost and the send, Alphan in the Deck and Graveyard to add.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, HAND),
    (12369277, HAND),
    (12369277, MZONE),
    (12369277, DECK),
    (ALPHAN, DECK),
    (ALPHAN, GRAVE),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
    Check::ResolvesEffect(aux::stringid(ID, 2)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_range(duel, LOCATION_HAND);
    e1.set_count_limit(duel, 1, ID);
    e1.set_cost(duel, spcost);
    e1.set_target(duel, sptg);
    e1.set_operation(duel, stock::special_summon_self::<POS_FACEUP_DEFENSE>);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_TOGRAVE);
    e2.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e2.set_code(duel, EVENT_SUMMON_SUCCESS);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_target(duel, tgtg);
    e2.set_operation(duel, tgop);
    c.register_effect(duel, e2);
    let e3 = e2.clone_effect(duel);
    e3.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    c.register_effect(duel, e3);
    let e4 = duel.create_effect(c);
    e4.set_description(duel, aux::stringid(ID, 2));
    e4.set_category(duel, CATEGORY_TOHAND + CATEGORY_SEARCH);
    e4.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e4.set_property(duel, EFFECT_FLAG_DELAY);
    e4.set_code(duel, EVENT_TO_GRAVE);
    e4.set_count_limit(duel, 1, (ID, 2));
    e4.set_target(duel, thtg);
    e4.set_operation(duel, thop);
    c.register_effect(duel, e4);
}

fn cfilter(c: CardId, duel: &Duel, tp: u8) -> bool {
    c.is_set_card(duel, &[SET_SUPER_QUANT])
        && c.is_attribute_except(duel, ATTRIBUTE_LIGHT)
        && (c.is_location(duel, LOCATION_HAND) || c.is_faceup(duel))
        && c.is_able_to_grave_as_cost(duel)
        && duel.get_mzone_count(tp, Some(c.into())) > 0
}

fn spcost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let (tp, c) = (a.tp, a.e.get_handler(duel));
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(
            |tc, duel| cfilter(tc, duel, tp),
            tp,
            LOCATION_HAND | LOCATION_MZONE,
            0,
            1,
            Some(c.into()),
        ));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TOGRAVE);
    let g = duel.select_matching_card(
        tp,
        |tc, duel| cfilter(tc, duel, tp),
        tp,
        LOCATION_HAND | LOCATION_MZONE,
        0,
        1,
        1,
        Some(c.into()),
    )?;
    duel.send_to_grave(g, REASON_COST)?;
    Ok(false)
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if chk == 0 {
        return Ok(c.is_can_be_special_summoned_full(duel, e, 0, tp, false, false, POS_FACEUP_DEFENSE, tp, 0xff));
    }
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, Some(c.into()), 1, tp, 0);
    Ok(false)
}

fn tgfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_SUPER_QUANT]) && c.is_monster(duel) && c.is_able_to_grave(duel)
}

fn tgtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    stock::exists_target(duel, a, chk, tgfilter, LOCATION_DECK, CATEGORY_TOGRAVE)
}

fn tgop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (tp, c) = (a.tp, a.e.get_handler(duel));
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TOGRAVE);
    let g = duel.select_matching_card(tp, tgfilter, tp, LOCATION_DECK, 0, 1, 1, None)?;
    if let Some(tc) = g.get_first()
        && duel.send_to_grave(tc, REASON_EFFECT)? > 0
        && tc.is_location(duel, LOCATION_GRAVE)
    {
        let e1 = duel.create_effect(c);
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_code(duel, EFFECT_CHANGE_LEVEL);
        e1.set_value(duel, tc.get_level(duel) as i32);
        e1.set_reset(duel, RESET_EVENT | RESETS_STANDARD_DISABLE);
        c.register_effect(duel, e1);
        let e2 = e1.clone_effect(duel);
        e2.set_code(duel, EFFECT_CHANGE_ATTRIBUTE);
        e2.set_value(duel, tc.get_attribute(duel) as i32);
        c.register_effect(duel, e2);
    }
    Ok(())
}

fn thfilter(c: CardId, duel: &Duel) -> bool {
    c.is_code(duel, ALPHAN) && c.is_able_to_hand(duel)
}

fn thtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    stock::exists_target(duel, a, chk, thfilter, LOCATION_DECK | LOCATION_GRAVE, CATEGORY_TOHAND)
}

fn thop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_ATOHAND);
    let g = duel.select_matching_card(
        tp,
        aux::necro_valley_filter(thfilter),
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
