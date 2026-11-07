// Chaos Emperor Dragon - Envoy of the End
use super::c72989439::{spcon1, spop1, sptg1};
use crate::prelude::*;
use crate::testing::{Check, GRAVE, HAND, MZONE, OPPONENT};

const ID: u32 = 82301904;

// Copies in the hand to Special Summon with a LIGHT and a DARK monster from the Graveyard, and on the field with cards of
// the opponent's to send.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, MZONE),
    (CARD_BLUEEYES_W_DRAGON, GRAVE),
    (CARD_DARK_MAGICIAN, GRAVE),
    (26202165, MZONE | OPPONENT),
    (26202165, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[Check::SpecialSummoned, Check::ResolvesEffect(aux::stringid(ID, 1))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    let e1 = duel.create_effect(c);
    e1.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE + EFFECT_FLAG_UNCOPYABLE);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_code(duel, EFFECT_SPSUMMON_CONDITION);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 0));
    e2.set_type(duel, EFFECT_TYPE_FIELD);
    e2.set_code(duel, EFFECT_SPSUMMON_PROC);
    e2.set_property(duel, EFFECT_FLAG_UNCOPYABLE);
    e2.set_range(duel, LOCATION_HAND);
    e2.set_condition(duel, spcon1);
    e2.set_target(duel, sptg1);
    e2.set_operation(duel, spop1);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 1));
    e3.set_category(duel, CATEGORY_TOGRAVE + CATEGORY_DAMAGE);
    e3.set_type(duel, EFFECT_TYPE_IGNITION);
    e3.set_range(duel, LOCATION_MZONE);
    e3.set_count_limit(duel, 1, 0);
    e3.set_cost(duel, sgcost);
    e3.set_target(duel, sgtg);
    e3.set_operation(duel, sgop);
    c.register_effect(duel, e3);
    duel.add_custom_activity_counter(ID, ACTIVITY_CHAIN, CounterFilter::Chain(|_, _, _, _| false));
}

fn sgcost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let (c, tp) = (a.e.get_handler(duel), a.tp);
    if chk == 0 {
        return Ok(duel.check_lp_cost(tp, 1000) && duel.get_custom_activity_count(ID, tp, ACTIVITY_CHAIN) == 0);
    }
    duel.pay_lp_cost(tp, 1000)?;
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_property(duel, EFFECT_FLAG_PLAYER_TARGET + EFFECT_FLAG_OATH);
    e1.set_code(duel, EFFECT_CANNOT_ACTIVATE);
    e1.set_target_range(duel, 1, 0);
    e1.set_value(duel, Value::EffectPlayer(|_, _, _, _| true));
    e1.set_reset(duel, RESET_PHASE | PHASE_END);
    duel.register_effect(e1, tp);
    let e2 = duel.create_effect(c);
    e2.set_property(duel, EFFECT_FLAG_PLAYER_TARGET + EFFECT_FLAG_CLIENT_HINT + EFFECT_FLAG_OATH);
    e2.set_description(duel, aux::stringid(ID, 2));
    e2.set_reset(duel, RESET_PHASE | PHASE_END);
    e2.set_target_range(duel, 1, 0);
    duel.register_effect(e2, tp);
    Ok(false)
}

fn sgtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(true);
    }
    let g = duel.get_field_group(tp, LOCATION_HAND | LOCATION_ONFIELD, LOCATION_HAND | LOCATION_ONFIELD);
    let dc = g.filter_count(duel, |c, duel| c.get_owner(duel) == 1 - tp && c.is_able_to_grave(duel), None);
    duel.set_operation_info(0, CATEGORY_TOGRAVE, Some(g), g.len() as u8, 0, 0);
    duel.set_operation_info(0, CATEGORY_DAMAGE, None, 0, 1 - tp, dc as i32 * 300);
    Ok(false)
}

fn sgop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    let g = duel.get_field_group(tp, LOCATION_HAND | LOCATION_ONFIELD, LOCATION_HAND | LOCATION_ONFIELD);
    duel.send_to_grave(g, REASON_EFFECT)?;
    let og = duel.get_operated_group();
    let ct = og.filter_count(
        duel,
        |c, duel| c.is_location(duel, LOCATION_GRAVE) && c.is_controler(duel, 1 - tp),
        None,
    ) as u32;
    if ct > 0 {
        duel.break_effect()?;
        duel.damage(1 - tp, ct * 300, REASON_EFFECT)?;
    }
    Ok(())
}
