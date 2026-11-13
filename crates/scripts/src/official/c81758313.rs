// Myriaday Dragon
use crate::prelude::*;
use crate::testing::{Check, HAND, MZONE, OPPONENT};

const ID: u32 = 81758313;
const COUNTER_MYRIAD: u16 = 0x21d;

pub const COUNTER_PLACE_LIST: &[u16] = &[COUNTER_MYRIAD];

// On the field, with Elfnote Lucina on the opponent's field to build an LP difference and Raigeki to destroy it.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (ID, MZONE),
    (13597785, MZONE | OPPONENT),
    (13597785, MZONE | OPPONENT),
    (12580477, HAND | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_counter_permit(duel, COUNTER_MYRIAD);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_COUNTER);
    e1.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e1.set_code(duel, EVENT_PHASE + PHASE_END);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_count_limit(duel, 1, 0);
    e1.set_cost(duel, Cost::pay_lp::<100>);
    e1.set_target(duel, |duel, a, chk, _| {
        let c = a.e.get_handler(duel);
        if chk == 0 {
            return Ok(c.is_can_add_counter(duel, COUNTER_MYRIAD, 1));
        }
        duel.set_operation_info(0, CATEGORY_COUNTER, Some(c.into()), 1, a.tp, COUNTER_MYRIAD.into());
        Ok(false)
    });
    e1.set_operation(duel, |duel, a| {
        let c = a.e.get_handler(duel);
        if c.is_relate_to_effect(duel, a.e) {
            c.add_counter(duel, COUNTER_MYRIAD, 1);
        }
        Ok(())
    });
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_COUNTER);
    e2.set_type(duel, EFFECT_TYPE_IGNITION);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_count_limit(duel, 1, 0);
    e2.set_target(duel, |duel, a, chk, _| {
        let c = a.e.get_handler(duel);
        let number_of_counters = lp_difference(duel, a.tp);
        if chk == 0 {
            return Ok(number_of_counters > 0 && c.is_can_add_counter(duel, COUNTER_MYRIAD, number_of_counters));
        }
        duel.set_operation_info(
            0,
            CATEGORY_COUNTER,
            Some(c.into()),
            number_of_counters as u8,
            a.tp,
            COUNTER_MYRIAD.into(),
        );
        Ok(false)
    });
    e2.set_operation(duel, |duel, a| {
        let c = a.e.get_handler(duel);
        if !c.is_relate_to_effect(duel, a.e) {
            return Ok(());
        }
        let number_of_counters = lp_difference(duel, a.tp);
        if number_of_counters > 0 {
            c.add_counter(duel, COUNTER_MYRIAD, number_of_counters);
        }
        Ok(())
    });
    c.register_effect(duel, e2);
    let e3a = duel.create_effect(c);
    e3a.set_type(duel, EFFECT_TYPE_SINGLE);
    e3a.set_property(duel, EFFECT_FLAG_SINGLE_RANGE);
    e3a.set_code(duel, EFFECT_UPDATE_ATTACK);
    e3a.set_range(duel, LOCATION_MZONE);
    e3a.set_condition(duel, |duel, a| a.e.get_handler(duel).get_counter(duel, COUNTER_MYRIAD) >= 10);
    e3a.set_value(duel, 10000);
    c.register_effect(duel, e3a);
    let e3b = e3a.clone_effect(duel);
    e3b.set_code(duel, EFFECT_UPDATE_DEFENSE);
    c.register_effect(duel, e3b);
    let e4 = duel.create_effect(c);
    e4.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_CONTINUOUS);
    e4.set_property(duel, EFFECT_FLAG_SINGLE_RANGE);
    e4.set_code(duel, EFFECT_DESTROY_REPLACE);
    e4.set_range(duel, LOCATION_MZONE);
    e4.set_target(duel, |duel, a, chk, _| {
        if chk == 0 {
            let c = a.e.get_handler(duel);
            return Ok(!c.is_reason(duel, REASON_REPLACE | REASON_RULE) && c.has_counter(duel, COUNTER_MYRIAD));
        }
        Ok(true)
    });
    e4.set_operation(duel, |duel, a| {
        a.e.get_handler(duel).remove_counter(duel, a.tp, COUNTER_MYRIAD, 1, REASON_EFFECT)?;
        Ok(())
    });
    c.register_effect(duel, e4);
}

fn lp_difference(duel: &Duel, tp: u8) -> u16 {
    ((duel.get_lp(tp) - duel.get_lp(1 - tp)).abs() / 1000) as u16
}
