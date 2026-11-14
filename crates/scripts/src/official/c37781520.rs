// Mermail Abyssleed
use super::c21954587::{spcost, spop, thcon, thfilter};
use crate::prelude::*;
use crate::testing::{Check, GRAVE, HAND, MZONE};

const ID: u32 = 37781520;

pub const LISTED_SERIES: &[u16] = &[SET_ABYSS, SET_MERMAIL];
// Atlantean Pikeman (WATER Normal) in the hand to discard, Mermail Abysspike on the field to release, and Abyss-scale
// of the Kraken in the GY to add.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (26976414, HAND),
    (26976414, HAND),
    (26976414, HAND),
    (58471134, HAND),
    (58471134, MZONE),
    (58471134, MZONE),
    (8719957, GRAVE),
    (63941169, GRAVE),
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
    e1.set_cost(duel, spcost::<3>);
    e1.set_target(duel, stock::special_summon_self_target_player_0);
    e1.set_operation(duel, spop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_TOHAND + CATEGORY_SEARCH);
    e2.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e2.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e2.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e2.set_condition(duel, thcon);
    e2.set_target(duel, thtg);
    e2.set_operation(duel, thop);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 2));
    e3.set_category(duel, CATEGORY_TOGRAVE);
    e3.set_property(duel, EFFECT_FLAG_PLAYER_TARGET);
    e3.set_type(duel, EFFECT_TYPE_IGNITION);
    e3.set_range(duel, LOCATION_MZONE);
    e3.set_count_limit(duel, 1, ID);
    e3.set_cost(duel, hdcost);
    e3.set_target(duel, hdtg);
    e3.set_operation(duel, hdop);
    c.register_effect(duel, e3);
}

fn thtg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_GRAVE) && chkc.is_controler(duel, tp) && thfilter(chkc, duel));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(thfilter, tp, LOCATION_GRAVE, 0, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_RTOHAND);
    let g = duel.select_target(tp, thfilter, tp, LOCATION_GRAVE, 0, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_TOHAND, g, 1, 0, 0);
    Ok(false)
}

fn thop(duel: &mut Duel, a: &Args) -> Script<()> {
    if let Some(tc) = duel.get_first_target()
        && tc.is_relate_to_effect(duel, a.e)
    {
        duel.send_to_hand(tc, PLAYER_NONE, REASON_EFFECT)?;
        duel.confirm_cards(1 - a.tp, tc)?;
    }
    Ok(())
}

fn costfilter(c: CardId, duel: &Duel) -> bool {
    c.is_position(duel, POS_FACEUP_ATTACK) && c.is_set_card(duel, &[SET_MERMAIL])
}

fn hdcost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let tp = a.tp;
    let c = a.e.get_handler(duel);
    if chk == 0 {
        return Ok(duel.check_release_group_cost(tp, costfilter, (1, 1), false, None, Some(c.into())));
    }
    let sg = duel.select_release_group_cost(tp, costfilter, (1, 1), false, None, Some(c.into()))?;
    duel.release(sg, REASON_COST)?;
    Ok(false)
}

fn hdtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(duel.get_field_group_count(1 - a.tp, LOCATION_HAND, 0) != 0);
    }
    duel.set_operation_info(0, CATEGORY_TOGRAVE, None, 1, 1 - a.tp, LOCATION_HAND as i32);
    Ok(false)
}

fn hdop(duel: &mut Duel, a: &Args) -> Script<()> {
    let g = duel.get_field_group(1 - a.tp, LOCATION_HAND, 0);
    if g.is_empty() {
        return Ok(());
    }
    let sg = g.random_select(duel, 1 - a.tp, 1);
    duel.send_to_grave(sg, REASON_EFFECT)?;
    Ok(())
}
