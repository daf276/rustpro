// Ghost Sister & Spooky Dogwood
use crate::prelude::*;
use crate::testing::{Check, HAND, OPPONENT};
use engine::constants::MSG_RECOVER;

const ID: u32 = 60643553;

// Copies in the hand to discard, with the opponent's Mathmech Sigma and Level 3 Psychic Wheeleder in the hand to Special Summon.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, HAND),
    (ID, HAND),
    (27182739, HAND | OPPONENT),
    (27182739, HAND | OPPONENT),
    (27182739, HAND | OPPONENT),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0)), Check::Message(MSG_RECOVER)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_RECOVER);
    e1.set_type(duel, EFFECT_TYPE_QUICK_O);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_hint_timing(duel, 0, TIMINGS_CHECK_MONSTER);
    e1.set_range(duel, LOCATION_HAND);
    e1.set_count_limit(duel, 1, ID);
    e1.set_condition(duel, |duel, _| duel.get_current_phase() < PHASE_END);
    e1.set_cost(duel, Cost::self_discard);
    e1.set_operation(duel, operation);
    c.register_effect(duel, e1);
}

fn operation(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    let e0 = duel.create_effect(c);
    e0.set_description(duel, aux::stringid(ID, 0));
    e0.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_CONTINUOUS);
    e0.set_code(duel, EVENT_PHASE + PHASE_END);
    e0.set_property(duel, EFFECT_FLAG_CLIENT_HINT);
    e0.set_count_limit(duel, 1, (ID, 1));
    e0.set_condition(duel, |duel, a| duel.get_flag_effect(a.tp, ID + 1) == 0);
    e0.set_operation(duel, damop);
    e0.set_reset(duel, RESET_PHASE | PHASE_END);
    duel.register_effect(e0, tp);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_CONTINUOUS + EFFECT_TYPE_FIELD);
    e1.set_property(duel, EFFECT_FLAG_DELAY);
    e1.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e1.set_condition(duel, reccon1);
    e1.set_operation(duel, recop1);
    e1.set_reset(duel, RESET_PHASE | PHASE_END);
    duel.register_effect(e1, tp);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_CONTINUOUS + EFFECT_TYPE_FIELD);
    e2.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e2.set_condition(duel, regcon);
    e2.set_operation(duel, regop);
    e2.set_reset(duel, RESET_PHASE | PHASE_END);
    duel.register_effect(e2, tp);
    let e3 = duel.create_effect(c);
    e3.set_type(duel, EFFECT_TYPE_CONTINUOUS + EFFECT_TYPE_FIELD);
    e3.set_code(duel, EVENT_CHAIN_SOLVED);
    e3.set_condition(duel, |duel, a| duel.get_flag_effect(a.tp, ID) > 0);
    e3.set_operation(duel, recop2);
    e3.set_reset(duel, RESET_PHASE | PHASE_END);
    duel.register_effect(e3, tp);
    e2.set_label_object(duel, Some(e3));
    Ok(())
}

fn filter(c: CardId, duel: &Duel, sp: u8) -> bool {
    if c.is_location(duel, LOCATION_MZONE) {
        c.is_faceup(duel) && c.is_type(duel, TYPE_EFFECT) && c.get_summon_player(duel) == sp
    } else {
        c.is_previous_position(duel, POS_FACEUP)
            && c.get_previous_type_on_field(duel) & TYPE_EFFECT != 0
            && c.is_previous_location(duel, LOCATION_MZONE)
            && c.get_summon_player(duel) == sp
    }
}

fn sum(c: CardId, duel: &Duel) -> i64 {
    i64::from(if c.is_location(duel, LOCATION_MZONE) {
        c.get_attack(duel)
    } else {
        c.get_previous_attack_on_field(duel)
    })
}

fn summoned_by_opponent(duel: &Duel, a: &Args) -> bool {
    let tp = a.tp;
    let ph = duel.get_current_phase();
    let matched =
        a.eg.is_some_and(|eg| eg.is_exists(duel, |c, duel| filter(c, duel, 1 - tp), 1, None));
    // Lua errors on a nil effect
    unported!(duel, matched && a.re.is_none());
    matched && (PHASE_MAIN1..=PHASE_MAIN2).contains(&ph)
}

fn reccon1(duel: &Duel, a: &Args) -> bool {
    summoned_by_opponent(duel, a)
        && a.re
            .is_some_and(|re| !re.is_has_type(duel, EFFECT_TYPE_ACTIONS) || re.is_has_type(duel, EFFECT_TYPE_CONTINUOUS))
}

fn regcon(duel: &Duel, a: &Args) -> bool {
    summoned_by_opponent(duel, a)
        && a.re
            .is_some_and(|re| re.is_has_type(duel, EFFECT_TYPE_ACTIONS) && !re.is_has_type(duel, EFFECT_TYPE_CONTINUOUS))
}

fn recover(duel: &mut Duel, tp: u8, amount: i64) -> Script<()> {
    duel.hint(HINT_CARD, 0, ID.into());
    if duel.recover(tp, amount as u32, REASON_EFFECT)? != 0 {
        duel.register_flag_effect(tp, ID + 1, RESET_PHASE | PHASE_END, 0, 1);
    }
    Ok(())
}

fn recop1(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    let g = a.eg.unwrap_or_default().filter(duel, |c, duel| filter(c, duel, 1 - tp), None);
    if !g.is_empty() {
        let total = g.get_sum(duel, sum);
        recover(duel, tp, total)?;
    }
    Ok(())
}

fn regop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    let g = a.eg.unwrap_or_default().filter(duel, |c, duel| filter(c, duel, 1 - tp), None);
    duel.register_flag_effect(tp, ID, RESET_CHAIN, 0, 1);
    if let Some(e3) = a.e.get_label_object(duel) {
        let total = g.get_sum(duel, sum) + e3.get_label(duel);
        e3.set_label(duel, total);
    }
    Ok(())
}

fn recop2(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    duel.reset_flag_effect(tp, ID);
    let rec = a.e.get_label(duel);
    a.e.set_label(duel, 0);
    recover(duel, tp, rec)
}

fn damop(duel: &mut Duel, a: &Args) -> Script<()> {
    duel.hint(HINT_CARD, 0, ID.into());
    let lp = duel.get_lp(a.tp);
    // Lua halves with a float, which the core rounds.
    duel.set_lp(a.tp, (f64::from(lp) / 2.0).round() as i32);
    Ok(())
}
