// Iron Core of Koa'ki Meiru
use crate::keys::{REPLACE_COUNT, REPLACE_MAX};
use crate::prelude::*;
use crate::testing::{Check, GRAVE, HAND};

const ID: u32 = 36623431;

pub const LISTED_SERIES: &[u16] = &[SET_KOAKI_MEIRU];
// In the Graveyard, with Koa'ki Meiru Guardian in the hand as the Koa'ki Meiru monster to discard.
pub const SUPPORT: &[(u32, u16)] = &[(ID, GRAVE), (ID, GRAVE), (45041488, HAND), (45041488, HAND)];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_TOHAND);
    e1.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e1.set_code(duel, EVENT_PREDRAW);
    e1.set_range(duel, LOCATION_GRAVE);
    e1.set_condition(duel, condition1);
    e1.set_target(duel, target1);
    e1.set_operation(duel, operation1);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_TOHAND);
    e2.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e2.set_code(duel, EVENT_PHASE | PHASE_DRAW);
    e2.set_count_limit(duel, 1, 0);
    e2.set_range(duel, LOCATION_GRAVE);
    e2.set_condition(duel, |duel, a| a.tp == duel.get_turn_player());
    e2.set_cost(duel, cost2);
    e2.set_target(duel, target2);
    e2.set_operation(duel, operation2);
    c.register_effect(duel, e2);
}

fn condition1(duel: &Duel, a: &Args) -> bool {
    a.tp == duel.get_turn_player() && duel.get_field_group_count(a.tp, LOCATION_DECK, 0) > 0 && duel.get_draw_count(a.tp) > 0
}

fn target1(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (c, tp) = (a.e.get_handler(duel), a.tp);
    if chk == 0 {
        return Ok(c.is_able_to_hand(duel));
    }
    let dt = duel.get_draw_count(tp);
    if dt != 0 {
        duel.set_script_state(REPLACE_COUNT, 0);
        duel.set_script_state(REPLACE_MAX, i64::from(dt));
        let e1 = duel.create_effect(c);
        e1.set_type(duel, EFFECT_TYPE_FIELD);
        e1.set_property(duel, EFFECT_FLAG_PLAYER_TARGET);
        e1.set_code(duel, EFFECT_DRAW_COUNT);
        e1.set_target_range(duel, 1, 0);
        e1.set_reset(duel, RESET_PHASE | PHASE_DRAW);
        e1.set_value(duel, 0);
        duel.register_effect(e1, tp);
    }
    duel.set_operation_info(0, CATEGORY_TOHAND, Some(c.into()), 1, 0, 0);
    Ok(false)
}

fn operation1(duel: &mut Duel, a: &Args) -> Script<()> {
    let (c, tp) = (a.e.get_handler(duel), a.tp);
    let count = duel.get_script_state(REPLACE_COUNT) + 1;
    duel.set_script_state(REPLACE_COUNT, count);
    if count <= duel.get_script_state(REPLACE_MAX) && c.is_relate_to_effect(duel, a.e) {
        duel.send_to_hand(c, PLAYER_NONE, REASON_EFFECT)?;
        duel.confirm_cards(1 - tp, c)?;
    }
    Ok(())
}

fn costfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_KOAKI_MEIRU]) && c.is_monster(duel) && c.is_able_to_grave_as_cost(duel)
}

fn cost2(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(costfilter, a.tp, LOCATION_HAND, 0, 1, None));
    }
    duel.discard_hand(a.tp, costfilter, 1, 1, REASON_COST)?;
    Ok(false)
}

fn target2(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let c = a.e.get_handler(duel);
    if chk == 0 {
        return Ok(c.is_able_to_hand(duel));
    }
    duel.set_operation_info(0, CATEGORY_TOHAND, Some(c.into()), 1, 0, 0);
    Ok(false)
}

fn operation2(duel: &mut Duel, a: &Args) -> Script<()> {
    let (c, tp) = (a.e.get_handler(duel), a.tp);
    if c.is_relate_to_effect(duel, a.e) {
        duel.send_to_hand(c, PLAYER_NONE, REASON_EFFECT)?;
        duel.confirm_cards(1 - tp, c)?;
    }
    Ok(())
}
