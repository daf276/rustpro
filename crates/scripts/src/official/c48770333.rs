// Thunder King, the Lightningstrike Kaiju
use crate::prelude::*;
use crate::testing::{Check, HAND, MZONE, OPPONENT};

const ID: u32 = 48770333;

pub const LISTED_SERIES: &[u16] = &[SET_KAIJU];
pub const COUNTER_LIST: &[u16] = &[COUNTER_KAIJU];

// In the hand with a copy on the opponent's field to Tribute, Kyoutou Waterfront for Kaiju Counters, and Raigeki and Dark Hole to fill it.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, MZONE | OPPONENT),
    (56111151, HAND),
    (12580477, HAND),
    (53129443, HAND),
    (26202165, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.set_unique_on_field(duel, 1, 0, |c, duel| c.is_set_card(duel, &[SET_KAIJU]), LOCATION_MZONE);
    aux::add_kaiju_procedure(duel, c);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 0));
    e3.set_type(duel, EFFECT_TYPE_IGNITION);
    e3.set_range(duel, LOCATION_MZONE);
    e3.set_count_limit(duel, 1, 0);
    e3.set_condition(duel, |duel, _| duel.is_able_to_enter_bp());
    e3.set_cost(duel, aux::kaiju_counter_cost::<3>);
    e3.set_operation(duel, atkop);
    c.register_effect(duel, e3);
}

fn atkop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_property(duel, EFFECT_FLAG_PLAYER_TARGET);
    e1.set_code(duel, EFFECT_CANNOT_ACTIVATE);
    e1.set_target_range(duel, 0, 1);
    e1.set_value(duel, 1);
    e1.set_reset(duel, RESET_PHASE | PHASE_END);
    duel.register_effect(e1, tp);
    if c.is_relate_to_effect(duel, e) {
        let e2 = duel.create_effect(c);
        e2.set_type(duel, EFFECT_TYPE_SINGLE);
        e2.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE);
        e2.set_code(duel, EFFECT_EXTRA_ATTACK_MONSTER);
        e2.set_value(duel, 2);
        e2.set_reset(duel, RESETS_STANDARD_PHASE_END);
        c.register_effect(duel, e2);
    }
    Ok(())
}
