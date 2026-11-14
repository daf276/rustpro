// Sangen Kaiho
use crate::prelude::*;
use crate::testing::{Check, GRAVE, HAND, MZONE, OPPONENT, SZONE};

const ID: u32 = 25388971;

pub const LISTED_SERIES: &[u16] = &[SET_TENPAI_DRAGON];
// Set to end the Main Phase with Tenpai Dragon Paidra alone against the opponent's Sangans, and in the GY with Paidra in the
// hand to summon once three attacks were declared.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, SZONE),
    (ID, GRAVE),
    (39931513, MZONE),
    (39931513, HAND),
    (26202165, MZONE | OPPONENT),
    (26202165, MZONE | OPPONENT),
    (26202165, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_hint_timing(duel, 0, TIMINGS_CHECK_MONSTER);
    e1.set_count_limit(duel, 1, ID);
    e1.set_condition(duel, phcon);
    e1.set_operation(duel, |duel, a| {
        let phase = duel.get_current_phase();
        duel.skip_phase(1 - a.tp, phase, RESET_PHASE | PHASE_END, 1);
        Ok(())
    });
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_DRAW + CATEGORY_SPECIAL_SUMMON);
    e2.set_type(duel, EFFECT_TYPE_QUICK_O);
    e2.set_code(duel, EVENT_FREE_CHAIN);
    e2.set_range(duel, LOCATION_GRAVE);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_hint_timing(duel, 0, TIMING_MAIN_END | TIMING_BATTLE_END | TIMINGS_CHECK_MONSTER_E);
    e2.set_condition(duel, |duel, _| duel.get_flag_effect(0, ID) + duel.get_flag_effect(1, ID) >= 3);
    e2.set_cost(duel, Cost::self_banish);
    e2.set_target(duel, drtg);
    e2.set_operation(duel, drop);
    c.register_effect(duel, e2);
    aux::global_check(duel, ID, |duel| {
        let ge1 = duel.create_effect(c);
        ge1.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_CONTINUOUS);
        ge1.set_code(duel, EVENT_ATTACK_ANNOUNCE);
        ge1.set_operation(duel, |duel, a| {
            duel.register_flag_effect(a.ep, ID, RESET_PHASE | PHASE_END, 0, 1);
            Ok(())
        });
        duel.register_effect(ge1, 0);
    });
}

fn phconfilter(c: CardId, duel: &Duel) -> bool {
    c.is_faceup(duel) && c.is_attribute(duel, ATTRIBUTE_FIRE) && c.is_race(duel, RACE_DRAGON)
}

fn phcon(duel: &Duel, a: &Args) -> bool {
    if !duel.is_main_phase() {
        return false;
    }
    let g = duel.get_field_group(a.tp, LOCATION_MZONE, 0);
    !g.is_empty() && g.filter_count(duel, phconfilter, None) == g.len() && duel.get_field_group_count(a.tp, 0, LOCATION_MZONE) > g.len()
}

fn drtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.is_player_can_draw(tp, 1));
    }
    duel.set_operation_info(0, CATEGORY_DRAW, None, 0, tp, 1);
    duel.set_possible_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, tp, LOCATION_HAND as i32);
    Ok(false)
}

fn drop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    if duel.draw(tp, 1, REASON_EFFECT)? == 0 {
        return Ok(());
    }
    let g = duel.get_matching_group(
        |c, duel| c.is_set_card(duel, &[SET_TENPAI_DRAGON]) && c.is_can_be_special_summoned(duel, e, 0, tp, false, false),
        tp,
        LOCATION_HAND,
        0,
        None,
    );
    let mut ct = (g.len() as i32).min(duel.get_location_count(tp, LOCATION_MZONE));
    if ct <= 0 || !duel.select_yes_no(tp, aux::stringid(ID, 2))? {
        return Ok(());
    }
    if duel.is_player_affected_by_effect(tp, CARD_BLUEEYES_SPIRIT).is_some() {
        ct = 1;
    }
    duel.shuffle_hand(tp)?;
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
    let sg = g.select(duel, tp, 1, ct as u8, None)?;
    if !sg.is_empty() {
        duel.break_effect()?;
        duel.special_summon(sg, 0, tp, tp, false, false, POS_FACEUP)?;
    }
    Ok(())
}
