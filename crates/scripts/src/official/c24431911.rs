// Tellusion the Magna Warrior
use crate::prelude::*;
use crate::testing::{Check, GRAVE, HAND, MZONE, OPPONENT, SZONE};

const ID: u32 = 24431911;
const SIGMA_PLUS: u32 = 51826619;
const SIGMA_MINUS: u32 = 87814728;

pub const LISTED_NAMES: &[u32] = &[SIGMA_PLUS, SIGMA_MINUS];
pub const LISTED_SERIES: &[u16] = &[SET_MAGNET_WARRIOR_SIGMA];

// Magnet Warrior Sigma Plus and Minus to banish (and summon back), and the opponent's Elfnote Lucinas and Mystical Space Typhoons to respond to.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, GRAVE),
    (SIGMA_PLUS, GRAVE),
    (SIGMA_PLUS, HAND),
    (SIGMA_MINUS, GRAVE),
    (SIGMA_MINUS, HAND),
    (13597785, MZONE | OPPONENT),
    (13597785, HAND | OPPONENT),
    (5318639, SZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::SpecialSummoned,
    Check::ResolvesEffect(aux::stringid(ID, 1)),
    Check::ResolvesEffect(aux::stringid(ID, 2)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    c.add_must_be_special_summoned(duel);
    let e0 = duel.create_effect(c);
    e0.set_description(duel, aux::stringid(ID, 0));
    e0.set_type(duel, EFFECT_TYPE_FIELD);
    e0.set_property(duel, EFFECT_FLAG_UNCOPYABLE + EFFECT_FLAG_CANNOT_DISABLE);
    e0.set_code(duel, EFFECT_SPSUMMON_PROC);
    e0.set_range(duel, LOCATION_GRAVE);
    e0.set_condition(duel, selfspcon);
    e0.set_target(duel, selfsptg);
    e0.set_operation(duel, selfspop);
    c.register_effect(duel, e0);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 1));
    e1.set_category(duel, CATEGORY_DESTROY + CATEGORY_CONTROL);
    e1.set_type(duel, EFFECT_TYPE_QUICK_O);
    e1.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e1.set_code(duel, EVENT_CHAINING);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_count_limit(duel, 1, 0);
    e1.set_condition(duel, |_, a| a.rp == 1 - a.tp);
    e1.set_target(duel, destg);
    e1.set_operation(duel, desop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 2));
    e2.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e2.set_type(duel, EFFECT_TYPE_QUICK_O);
    e2.set_code(duel, EVENT_FREE_CHAIN);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_condition(duel, |duel, a| duel.is_turn_player(1 - a.tp));
    e2.set_cost(duel, Cost::self_tribute);
    e2.set_target(duel, rmsptg);
    e2.set_operation(duel, rmspop);
    e2.set_hint_timing(duel, 0, TIMING_STANDBY_PHASE | TIMING_MAIN_END | TIMINGS_CHECK_MONSTER_E);
    c.register_effect(duel, e2);
}

fn selfspcostfilter(c: CardId, duel: &Duel) -> bool {
    (c.is_code(duel, SIGMA_PLUS) || c.is_code(duel, SIGMA_MINUS))
        && (c.is_faceup(duel) || !c.is_on_field(duel))
        && c.is_able_to_remove_as_cost(duel)
        && aux::sp_elim_filter(c, duel, true, true)
}

fn rescon(duel: &Duel, sg: CardSet, tp: u8) -> (bool, bool) {
    let res = duel.get_mzone_count(tp, Some(sg)) > 0
        && sg.is_exists(duel, |c, duel| c.is_code(duel, SIGMA_PLUS), 1, None)
        && sg.is_exists(duel, |c, duel| c.is_code(duel, SIGMA_MINUS), 1, None);
    (res, false)
}

fn cost_group(duel: &Duel, tp: u8) -> CardSet {
    duel.get_matching_group(selfspcostfilter, tp, LOCATION_HAND | LOCATION_MZONE | LOCATION_GRAVE, 0, None)
}

fn selfspcon(duel: &Duel, a: &Args) -> bool {
    if a.c.is_none() {
        return true;
    }
    let tp = a.e.get_handler_player(duel);
    let g = cost_group(duel, tp);
    g.len() >= 2
        && duel.get_mzone_count(tp, Some(g)) > 0
        && aux::select_unselect_group_check(duel, g, 2, 2, &|duel, sg, _, _| rescon(duel, sg, tp))
}

fn selfsptg(duel: &mut Duel, a: &Args, _: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    let g = cost_group(duel, tp);
    let sg = aux::select_unselect_group_full(
        duel,
        g,
        2,
        2,
        &|duel, sg, _, _| rescon(duel, sg, tp),
        tp,
        HINTMSG_REMOVE,
        None,
        true,
    )?;
    if !sg.is_empty() {
        a.e.set_label_object_group(duel, Some(sg));
        return Ok(true);
    }
    Ok(false)
}

fn selfspop(duel: &mut Duel, a: &Args) -> Script<()> {
    if let Some(sg) = a.e.get_label_object_group(duel) {
        duel.remove(sg, POS_FACEUP, REASON_COST)?;
    }
    Ok(())
}

fn destg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_controler(duel, 1 - tp) && chkc.is_location(duel, LOCATION_MZONE));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(aux::TRUE, tp, 0, LOCATION_MZONE, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_DESTROY);
    let Some(tc) = duel
        .select_target(tp, aux::TRUE, tp, 0, LOCATION_MZONE, 1, 1, None)?
        .and_then(|g| g.get_first())
    else {
        return Ok(false);
    };
    if tc.is_attribute(duel, ATTRIBUTE_EARTH) && tc.is_faceup(duel) {
        a.e.set_label(duel, 100);
        duel.set_possible_operation_info(0, CATEGORY_DESTROY, Some(tc.into()), 1, tp, 0);
        duel.set_possible_operation_info(0, CATEGORY_CONTROL, Some(tc.into()), 1, tp, 0);
    } else {
        a.e.set_label(duel, 0);
        duel.set_operation_info(0, CATEGORY_DESTROY, Some(tc.into()), 1, tp, 0);
    }
    Ok(false)
}

fn desop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    let Some(tc) = duel.get_first_target() else {
        return Ok(());
    };
    if !tc.is_relate_to_effect(duel, a.e) {
        return Ok(());
    }
    let earth_chk = a.e.get_label(duel) == 100;
    if earth_chk && tc.is_controler_can_be_changed(duel) && duel.select_yes_no(tp, aux::stringid(ID, 3))? {
        duel.get_control(tc, tp)?;
    } else {
        duel.destroy(tc, REASON_EFFECT)?;
    }
    Ok(())
}

fn rmspfilter(e: EffectId, tp: u8) -> impl Fn(CardId, &Duel) -> bool {
    move |c, duel| {
        c.is_set_card(duel, &[SET_MAGNET_WARRIOR_SIGMA]) && c.is_faceup(duel) && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
    }
}

fn rmsptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        let c = a.e.get_handler(duel);
        return Ok(duel.get_mzone_count(tp, Some(c.into())) >= 2
            && duel.is_player_affected_by_effect(tp, CARD_BLUEEYES_SPIRIT).is_none()
            && duel.is_existing_matching_card(rmspfilter(a.e, tp), tp, LOCATION_REMOVED, 0, 2, None));
    }
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 2, tp, LOCATION_REMOVED as i32);
    Ok(false)
}

fn rmspop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    if duel.get_location_count(tp, LOCATION_MZONE) < 2 || duel.is_player_affected_by_effect(tp, CARD_BLUEEYES_SPIRIT).is_some() {
        return Ok(());
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
    let g = duel.select_matching_card(tp, rmspfilter(a.e, tp), tp, LOCATION_REMOVED, 0, 2, 2, None)?;
    if g.len() == 2 {
        duel.special_summon(g, 0, tp, tp, false, false, POS_FACEUP)?;
    }
    Ok(())
}
