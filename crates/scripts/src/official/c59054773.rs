// Ignister A.I.Land
use crate::prelude::*;
use crate::testing::{Check, GRAVE, HAND};

const ID: u32 = 59054773;

pub const LISTED_SERIES: &[u16] = &[SET_IGNISTER];

const ATTR_LIST: u64 = (ID as u64) << 16;

// Activated, with Ignister monsters of Level 4 or less of several Attributes in the hand to Summon (it needs no monster in the Main Monster Zones), and in the
// Graveyard to banish when it Sets itself.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, GRAVE),
    (3723262, HAND),
    (3723262, HAND),
    (30118811, HAND),
    (42429678, HAND),
    (42429678, GRAVE),
    (30118811, GRAVE),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 0));
    e2.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e2.set_type(duel, EFFECT_TYPE_IGNITION);
    e2.set_range(duel, LOCATION_FZONE);
    e2.set_condition(duel, spcon);
    e2.set_target(duel, sptg);
    e2.set_operation(duel, spop);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 1));
    e3.set_category(duel, CATEGORY_SET);
    e3.set_type(duel, EFFECT_TYPE_IGNITION);
    e3.set_range(duel, LOCATION_GRAVE);
    e3.set_count_limit(duel, 1, ID);
    e3.set_cost(duel, setcost);
    e3.set_target(duel, settg);
    e3.set_operation(duel, setop);
    c.register_effect(duel, e3);
    aux::global_check(duel, ID, |duel| {
        aux::add_values_reset(duel, |duel| {
            duel.set_script_state(ATTR_LIST, 0);
            duel.set_script_state(ATTR_LIST | 1, 0);
        });
    });
}

fn spcon(duel: &Duel, a: &Args) -> bool {
    !duel.is_existing_matching_card(|c, duel| c.get_sequence(duel) < 5, a.tp, LOCATION_MZONE, 0, 1, None)
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_set_card(duel, &[SET_IGNISTER])
        && c.is_level_below(duel, 4)
        && duel.get_script_state(ATTR_LIST | u64::from(tp)) & i64::from(c.get_attribute(duel)) == 0
        && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    stock::special_summon_target(duel, a, chk, |c, duel| spfilter(c, duel, a.e, a.tp), LOCATION_HAND)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp, c) = (a.e, a.tp, a.e.get_handler(duel));
    if !c.is_relate_to_effect(duel, e) {
        return Ok(());
    }
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_code(duel, EFFECT_CANNOT_SPECIAL_SUMMON);
    e1.set_property(duel, EFFECT_FLAG_PLAYER_TARGET + EFFECT_FLAG_CLIENT_HINT);
    e1.set_description(duel, aux::stringid(ID, 2));
    e1.set_target_range(duel, 1, 0);
    e1.set_target_card(duel, |duel, _, c| !c.is_race(duel, RACE_CYBERSE));
    e1.set_reset(duel, RESET_PHASE | PHASE_END);
    duel.register_effect(e1, tp);
    if duel.get_location_count(tp, LOCATION_MZONE) <= 0 {
        return Ok(());
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
    let g = duel.select_matching_card(tp, |c, duel| spfilter(c, duel, e, tp), tp, LOCATION_HAND, 0, 1, 1, None)?;
    let Some(sc) = g.get_first() else { return Ok(()) };
    let attr = sc.get_attribute(duel);
    duel.special_summon(g, 0, tp, tp, false, false, POS_FACEUP)?;
    let key = ATTR_LIST | u64::from(tp);
    duel.set_script_state(key, duel.get_script_state(key) | i64::from(attr));
    for string in aux::get_attribute_strings(attr) {
        c.register_flag_effect_full(duel, 0, RESETS_STANDARD_PHASE_END, EFFECT_FLAG_CLIENT_HINT, 1, 0, string);
    }
    Ok(())
}

fn rmfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_IGNISTER])
        && c.is_monster(duel)
        && c.is_able_to_remove_as_cost(duel)
        && aux::sp_elim_filter(c, duel, true, false)
}

fn setcost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let (c, tp) = (a.e.get_handler(duel), a.tp);
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(rmfilter, tp, LOCATION_MZONE | LOCATION_GRAVE, 0, 1, Some(c.into())));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_REMOVE);
    let g = duel.select_matching_card(tp, rmfilter, tp, LOCATION_MZONE | LOCATION_GRAVE, 0, 1, 1, Some(c.into()))?;
    duel.remove(g, POS_FACEUP, REASON_COST)?;
    Ok(false)
}

fn settg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let c = a.e.get_handler(duel);
    if chk == 0 {
        return Ok(c.is_ssetable(duel));
    }
    duel.set_operation_info(0, CATEGORY_LEAVE_GRAVE, Some(c.into()), 1, 0, 0);
    Ok(false)
}

fn setop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (c, tp) = (a.e.get_handler(duel), a.tp);
    if c.is_relate_to_effect(duel, a.e) && c.is_ssetable(duel) {
        duel.sset(tp, c)?;
    }
    Ok(())
}
