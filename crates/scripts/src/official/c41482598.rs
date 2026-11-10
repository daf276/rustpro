// Mirage of Nightmare
use crate::prelude::*;
use crate::testing::{Check, HAND};

const ID: u32 = 41482598;

pub const SUPPORT: &[(u32, u16)] = &[(ID, HAND)];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_target(duel, clear);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 0));
    e2.set_category(duel, CATEGORY_DRAW);
    e2.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_F);
    e2.set_range(duel, LOCATION_SZONE);
    e2.set_count_limit(duel, 1, 0);
    e2.set_code(duel, EVENT_PHASE | PHASE_STANDBY);
    e2.set_condition(duel, |duel, a| {
        duel.is_turn_player(1 - a.tp) && duel.get_field_group_count(a.tp, LOCATION_HAND, 0) < 4
    });
    e2.set_target(duel, drtg);
    e2.set_operation(duel, drop);
    e2.set_label_object(duel, Some(e1));
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 1));
    e3.set_category(duel, CATEGORY_HANDES);
    e3.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_F);
    e3.set_range(duel, LOCATION_SZONE);
    e3.set_count_limit(duel, 1, 0);
    e3.set_code(duel, EVENT_PHASE | PHASE_STANDBY);
    e3.set_condition(duel, dccon);
    e3.set_target(duel, dctg);
    e3.set_operation(duel, dcop);
    e3.set_label_object(duel, Some(e1));
    c.register_effect(duel, e3);
}

fn clear(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(true);
    }
    a.e.set_label(duel, 0);
    Ok(false)
}

fn drtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(true);
    }
    let ht = duel.get_field_group_count(tp, LOCATION_HAND, 0) as i32;
    duel.set_operation_info(0, CATEGORY_DRAW, None, 0, tp, 4 - ht);
    Ok(false)
}

fn drop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    if !e.get_handler(duel).is_relate_to_effect(duel, e) {
        return Ok(());
    }
    let ht = duel.get_field_group_count(tp, LOCATION_HAND, 0);
    let Some(de) = e.get_label_object(duel) else { return Ok(()) };
    if ht < 4 {
        duel.draw(tp, 4 - ht as u32, REASON_EFFECT)?;
        de.set_label(duel, 4 - ht as i64);
    } else {
        de.set_label(duel, 0);
    }
    Ok(())
}

fn dccon(duel: &Duel, a: &Args) -> bool {
    duel.is_turn_player(a.tp) && a.e.get_label_object(duel).is_some_and(|de| de.get_label(duel) != 0)
}

fn dctg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if chk == 0 {
        return Ok(true);
    }
    let Some(de) = e.get_label_object(duel) else { return Ok(false) };
    let label = de.get_label(duel);
    e.set_label(duel, label);
    de.set_label(duel, 0);
    duel.set_operation_info(0, CATEGORY_HANDES, None, 0, tp, label as i32);
    Ok(false)
}

fn dcop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    if !e.get_handler(duel).is_relate_to_effect(duel, e) {
        return Ok(());
    }
    let g = duel.get_field_group(tp, LOCATION_HAND, 0);
    let sg = g.random_select(duel, tp, e.get_label(duel) as usize);
    duel.send_to_grave(sg, REASON_EFFECT | REASON_DISCARD)?;
    Ok(())
}
