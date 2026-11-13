// Symph Amplifire
use crate::prelude::*;
use crate::testing::{Check, HAND, MZONE, OPPONENT};
use engine::constants::{MSG_ADD_COUNTER, MSG_REMOVE_COUNTER};

const ID: u32 = 75304793;

pub const COUNTER_PLACE_LIST: &[u16] = &[0x35];
const COUNTER_SYMPH: u16 = 0x35;

// Symphonic Warrior Drumss and Piaano on the field to chain their own effects for counters, with Dark Hole for the opponent's side.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (94331452, MZONE),
    (94331452, MZONE),
    (31826057, MZONE),
    (31826057, MZONE),
    (68933343, MZONE),
    (68933343, MZONE),
    (94331452, HAND),
    (31826057, HAND),
    (68933343, HAND),
    (53129443, HAND | OPPONENT),
];
pub const CHECKS: &[Check] = &[Check::Message(MSG_ADD_COUNTER), Check::Message(MSG_REMOVE_COUNTER)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_counter_permit(duel, COUNTER_SYMPH);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_CONTINUOUS);
    e2.set_code(duel, EVENT_CHAIN_SOLVING);
    e2.set_range(duel, LOCATION_FZONE);
    e2.set_property(duel, EFFECT_FLAG_DELAY);
    e2.set_condition(duel, ctcon);
    e2.set_operation(duel, ctop);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_type(duel, EFFECT_TYPE_FIELD);
    e3.set_code(duel, EFFECT_UPDATE_ATTACK);
    e3.set_range(duel, LOCATION_FZONE);
    e3.set_target_range(duel, LOCATION_MZONE, LOCATION_MZONE);
    e3.set_target_card(duel, |duel, _, c| c.is_set_card(duel, &[SET_SYMPHONIC_WARRIOR]));
    e3.set_value(duel, Value::Card(atkval));
    c.register_effect(duel, e3);
    let e4 = duel.create_effect(c);
    e4.set_type(duel, EFFECT_TYPE_IGNITION);
    e4.set_range(duel, LOCATION_FZONE);
    e4.set_count_limit(duel, 1, 0);
    e4.set_target(duel, target);
    e4.set_operation(duel, operation);
    c.register_effect(duel, e4);
}

fn ctcon(duel: &Duel, a: &Args) -> bool {
    a.re.is_some_and(|re| re.get_handler(duel).is_set_card(duel, &[SET_SYMPHONIC_WARRIOR]) && !re.is_has_type(duel, EFFECT_TYPE_ACTIVATE))
}

fn ctop(duel: &mut Duel, a: &Args) -> Script<()> {
    a.e.get_handler(duel).add_counter(duel, COUNTER_SYMPH, 1);
    Ok(())
}

fn atkval(duel: &Duel, e: EffectId, _: CardId) -> i32 {
    i32::from(e.get_handler(duel).get_counter(duel, COUNTER_SYMPH)) * 100
}

fn filter(c: CardId, duel: &Duel) -> bool {
    c.is_faceup(duel) && c.is_set_card(duel, &[SET_SYMPHONIC_WARRIOR])
}

fn rmfilter(c: CardId, duel: &Duel) -> bool {
    c.is_able_to_remove(duel) && aux::sp_elim_filter(c, duel, false, true)
}

fn target(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let ct = duel.get_matching_group_count(filter, tp, LOCATION_ONFIELD, LOCATION_ONFIELD, None) as i32;
    let b1 = duel.is_can_remove_counter(tp, 1, 0, COUNTER_SYMPH, 5, REASON_COST);
    let b2 = duel.is_can_remove_counter(tp, 1, 0, COUNTER_SYMPH, 7, REASON_COST)
        && duel.is_existing_matching_card(rmfilter, tp, 0, LOCATION_ONFIELD | LOCATION_GRAVE, 1, None);
    if chk == 0 {
        return Ok(ct > 0 && (b1 || b2));
    }
    let op = if b1 && b2 {
        duel.select_option(tp, &[aux::stringid(ID, 0), aux::stringid(ID, 1)])?
    } else if b1 {
        duel.select_option(tp, &[aux::stringid(ID, 0)])?
    } else {
        duel.select_option(tp, &[aux::stringid(ID, 1)])? + 1
    };
    e.set_label(duel, i64::from(op));
    if op == 0 {
        e.set_category(duel, CATEGORY_DAMAGE);
        duel.remove_counter(tp, 1, 0, COUNTER_SYMPH, 5, REASON_COST)?;
        duel.set_target_player(1 - tp);
        duel.set_operation_info(0, CATEGORY_DAMAGE, None, 0, 1 - tp, ct * 300);
    } else {
        e.set_category(duel, CATEGORY_REMOVE);
        duel.remove_counter(tp, 1, 0, COUNTER_SYMPH, 7, REASON_COST)?;
        duel.set_operation_info(0, CATEGORY_REMOVE, None, 1, 0, (LOCATION_ONFIELD | LOCATION_GRAVE) as i32);
    }
    Ok(false)
}

fn operation(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    if !e.get_handler(duel).is_relate_to_effect(duel, e) {
        return Ok(());
    }
    let ct = duel.get_matching_group_count(filter, tp, LOCATION_ONFIELD, LOCATION_ONFIELD, None) as u32;
    if ct == 0 {
        return Ok(());
    }
    if e.get_label(duel) == 0 {
        let p: u8 = duel.get_chain_info(0, CHAININFO_TARGET_PLAYER);
        duel.damage(p, ct * 300, REASON_EFFECT)?;
    } else {
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_REMOVE);
        let g = duel.select_matching_card(tp, rmfilter, tp, 0, LOCATION_ONFIELD | LOCATION_GRAVE, 1, ct as u8, None)?;
        if !g.is_empty() {
            duel.remove(g, POS_FACEUP, REASON_EFFECT)?;
        }
    }
    Ok(())
}
