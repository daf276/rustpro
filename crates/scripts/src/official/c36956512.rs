// Gadarla, the Mystery Dust Kaiju
use crate::prelude::*;
use crate::testing::{Check, HAND, MZONE, OPPONENT};

const ID: u32 = 36956512;

pub const COUNTER_LIST: &[u16] = &[COUNTER_KAIJU];

// A copy in hand, one on the opponent's field to Tribute, Kyoutou Waterfront for Kaiju Counters, and Raigeki and Dark Hole to fill it.
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
    aux::add_kaiju_procedure(duel, c);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 0));
    e3.set_category(duel, CATEGORY_ATKCHANGE + CATEGORY_DEFCHANGE);
    e3.set_type(duel, EFFECT_TYPE_QUICK_O);
    e3.set_code(duel, EVENT_FREE_CHAIN);
    e3.set_range(duel, LOCATION_MZONE);
    e3.set_property(duel, EFFECT_FLAG_DAMAGE_STEP);
    e3.set_hint_timing(duel, TIMING_DAMAGE_STEP, TIMING_DAMAGE_STEP | TIMINGS_CHECK_MONSTER);
    e3.set_count_limit(duel, 1, 0);
    e3.set_condition(duel, aux::stat_change_damage_step_condition);
    e3.set_cost(duel, aux::kaiju_counter_cost::<3>);
    e3.set_target(duel, atktg);
    e3.set_operation(duel, atkop);
    c.register_effect(duel, e3);
}

fn atktg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let c = a.e.get_handler(duel);
    Ok(chk == 0 && duel.is_existing_matching_card(CardId::is_faceup, a.tp, LOCATION_MZONE, LOCATION_MZONE, 1, Some(c.into())))
}

fn atkop(duel: &mut Duel, a: &Args) -> Script<()> {
    let c = a.e.get_handler(duel);
    for tc in duel.get_matching_group(CardId::is_faceup, a.tp, LOCATION_MZONE, LOCATION_MZONE, Some(c.into())) {
        let (atk, def) = (f64::from(tc.get_attack(duel)), f64::from(tc.get_defense(duel)));
        for (code, value) in [(EFFECT_SET_ATTACK_FINAL, atk), (EFFECT_SET_DEFENSE_FINAL, def)] {
            let e = duel.create_effect(c);
            e.set_type(duel, EFFECT_TYPE_SINGLE);
            e.set_code(duel, code);
            e.set_value(duel, (value / 2.0).round() as i32);
            e.set_reset(duel, RESET_EVENT | RESETS_STANDARD);
            tc.register_effect(duel, e);
        }
    }
    Ok(())
}
