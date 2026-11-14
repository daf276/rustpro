// Audhumla, Progenitor of the Frozen Expanse
use crate::prelude::*;
use crate::testing::{Check, HAND, MZONE, OPPONENT, SZONE};

const ID: u32 = 101402101;
const DRAW_COUNT: u8 = 0;

// In the hand and on the field with Set Mirror Forces to send, a WATER monster (Psychic Kappa) to summon, and the opponent's
// Raigeki to answer.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, MZONE),
    (ID, MZONE),
    (44095762, SZONE),
    (44095762, SZONE),
    (44095762, SZONE),
    (7892180, HAND),
    (12580477, HAND | OPPONENT),
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
    e1.set_count_limit(duel, 1, (ID, 0));
    e1.set_condition(duel, |duel, a| {
        duel.get_field_group_count(a.tp, LOCATION_ONFIELD, LOCATION_ONFIELD) >= 4
    });
    e1.set_target(duel, |duel, a, chk, _| {
        let (e, tp) = (a.e, a.tp);
        let c = e.get_handler(duel);
        if chk == 0 {
            return Ok(duel.get_location_count(tp, LOCATION_MZONE) > 0 && c.is_can_be_special_summoned(duel, e, 0, tp, false, false));
        }
        duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, Some(c.into()), 1, tp, 0);
        Ok(false)
    });
    e1.set_operation(duel, |duel, a| {
        let c = a.e.get_handler(duel);
        if c.is_relate_to_effect(duel, a.e) {
            duel.special_summon(c, 0, a.tp, a.tp, false, false, POS_FACEUP)?;
        }
        Ok(())
    });
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_DRAW + CATEGORY_SPECIAL_SUMMON);
    e2.set_type(duel, EFFECT_TYPE_IGNITION);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_cost(duel, drcost);
    e2.set_target(duel, drtg);
    e2.set_operation(duel, drop);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 2));
    e3.set_category(duel, CATEGORY_POSITION + CATEGORY_SET);
    e3.set_type(duel, EFFECT_TYPE_QUICK_O);
    e3.set_code(duel, EVENT_CHAINING);
    e3.set_range(duel, LOCATION_MZONE);
    e3.set_count_limit(duel, 1, (ID, 2));
    e3.set_condition(duel, |_, a| a.ep == 1 - a.tp);
    e3.set_target(duel, postg);
    e3.set_operation(duel, posop);
    c.register_effect(duel, e3);
}

fn costfilter(c: CardId, duel: &Duel) -> bool {
    c.is_spell_trap(duel) && c.is_able_to_grave_as_cost(duel)
}

fn drcost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(costfilter, tp, LOCATION_ONFIELD, 0, 1, None));
    }
    let max_count = if duel.is_player_can_draw(tp, 2) { 2 } else { 1 };
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TOGRAVE);
    let g = duel.select_matching_card(tp, costfilter, tp, LOCATION_ONFIELD, 0, 1, max_count, None)?;
    e.set_chain_data(duel, DRAW_COUNT, g.len() as i64);
    duel.send_to_grave(g, REASON_COST)?;
    Ok(false)
}

fn drtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if chk == 0 {
        return Ok(duel.is_player_can_draw(tp, 1));
    }
    let draw_count = e.get_chain_data(duel, DRAW_COUNT).unwrap_or(0);
    duel.set_operation_info(0, CATEGORY_DRAW, None, 0, tp, draw_count as i32);
    duel.set_possible_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, tp, LOCATION_HAND as i32);
    Ok(false)
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_attribute(duel, ATTRIBUTE_WATER) && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
}

fn drop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let draw_count = e.get_chain_data(duel, DRAW_COUNT).unwrap_or(0);
    if duel.draw(tp, draw_count as u32, REASON_EFFECT)? == draw_count as i32
        && duel.get_location_count(tp, LOCATION_MZONE) > 0
        && duel.is_existing_matching_card(|c, duel| spfilter(c, duel, e, tp), tp, LOCATION_HAND, 0, 1, None)
    {
        if !duel.select_yes_no(tp, aux::stringid(ID, 3))? {
            return duel.shuffle_hand(tp);
        }
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
        let g = duel.select_matching_card(tp, |c, duel| spfilter(c, duel, e, tp), tp, LOCATION_HAND, 0, 1, 1, None)?;
        if !g.is_empty() {
            duel.break_effect()?;
            duel.special_summon(g, 0, tp, tp, false, false, POS_FACEUP)?;
        }
    }
    Ok(())
}

fn postg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let c = a.e.get_handler(duel);
    if chk == 0 {
        return Ok(c.is_can_turn_set(duel));
    }
    duel.set_operation_info(0, CATEGORY_POSITION, Some(c.into()), 1, a.tp, POS_FACEDOWN_DEFENSE as i32);
    Ok(false)
}

fn posop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    let c = a.e.get_handler(duel);
    if c.is_relate_to_effect(duel, a.e) {
        duel.change_position(c, POS_FACEDOWN_DEFENSE)?;
    }
    aux::register_client_hint(duel, c, 0, tp, 1, 0, Some(aux::stringid(ID, 4)));
    let e1a = duel.create_effect(c);
    e1a.set_type(duel, EFFECT_TYPE_FIELD);
    e1a.set_property(duel, EFFECT_FLAG_SET_AVAILABLE);
    e1a.set_code(duel, EFFECT_INDESTRUCTABLE_EFFECT);
    e1a.set_target_range(duel, LOCATION_ONFIELD, 0);
    e1a.set_target_card(duel, |duel, _, c| c.is_facedown(duel));
    e1a.set_value(duel, Value::EffectPlayer(aux::indoval));
    e1a.set_reset(duel, RESET_PHASE | PHASE_END);
    duel.register_effect(e1a, tp);
    let e1b = e1a.clone_effect(duel);
    e1b.set_property(duel, EFFECT_FLAG_SET_AVAILABLE + EFFECT_FLAG_IGNORE_IMMUNE);
    e1b.set_code(duel, EFFECT_CANNOT_BE_EFFECT_TARGET);
    e1b.set_value(duel, Value::EffectPlayer(aux::tgoval));
    duel.register_effect(e1b, tp);
    Ok(())
}
