// Fiendsmith's Sanct
use crate::prelude::*;
use crate::testing::{Check, GRAVE, HAND, MZONE, OPPONENT};

const ID: u32 = 35552985;

pub const LISTED_NAMES: &[u32] = &[ID + 1];

// In the hand with Fiendsmith Engraver on the field, destroyed by the opponent's Raigeki, for the copy in the Graveyard to Set.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, HAND),
    (ID, GRAVE),
    (60764609, MZONE),
    (60764609, MZONE),
    (12580477, HAND | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON + CATEGORY_TOKEN);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_hint_timing(duel, 0, TIMING_MAIN_END | TIMINGS_CHECK_MONSTER_E);
    e1.set_count_limit(duel, 1, ID);
    e1.set_condition(duel, condition);
    e1.set_target(duel, target);
    e1.set_operation(duel, activate);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_SET);
    e2.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e2.set_property_full(duel, EFFECT_FLAG_DELAY, EFFECT_FLAG2_CHECK_SIMULTANEOUS);
    e2.set_code(duel, EVENT_DESTROYED);
    e2.set_range(duel, LOCATION_GRAVE);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_condition(duel, setcon);
    e2.set_target(duel, settg);
    e2.set_operation(duel, setop);
    c.register_effect(duel, e2);
}

fn confilter(c: CardId, duel: &Duel) -> bool {
    c.is_attribute(duel, ATTRIBUTE_LIGHT) && c.is_race(duel, RACE_FIEND)
}

fn condition(duel: &Duel, a: &Args) -> bool {
    let g = duel.get_matching_group(CardId::is_faceup, a.tp, LOCATION_MZONE, 0, None);
    g.is_empty() || g.filter_count(duel, confilter, None) == g.len()
}

fn can_summon_token(duel: &mut Duel, tp: u8) -> bool {
    let stats = MonsterStats::new(SET_FIENDSMITH, TYPES_TOKEN, 0, 0, 1, RACE_FIEND, ATTRIBUTE_LIGHT);
    duel.get_location_count(tp, LOCATION_MZONE) > 0 && duel.is_player_can_special_summon_monster(tp, ID + 1, stats)
}

fn target(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(can_summon_token(duel, tp));
    }
    duel.set_operation_info(0, CATEGORY_TOKEN, None, 1, 0, 0);
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, tp, 0);
    Ok(false)
}

fn activate(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    if can_summon_token(duel, tp)
        && let Some(token) = duel.create_token(tp, ID + 1)
    {
        duel.special_summon(token, 0, tp, tp, false, false, POS_FACEUP)?;
    }
    let c = a.e.get_handler(duel);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_property(duel, EFFECT_FLAG_IGNORE_IMMUNE);
    e1.set_code(duel, EFFECT_CANNOT_ATTACK_ANNOUNCE);
    e1.set_target_range(duel, LOCATION_MZONE, 0);
    e1.set_target_card(duel, |duel, _, c| !c.is_race(duel, RACE_FIEND));
    e1.set_reset(duel, RESET_PHASE | PHASE_END);
    duel.register_effect(e1, tp);
    aux::register_client_hint(duel, c, 0, tp, 1, 0, Some(aux::stringid(ID, 2)));
    Ok(())
}

fn setconfilter(c: CardId, duel: &Duel, tp: u8) -> bool {
    c.is_previous_controler(duel, tp)
        && c.is_previous_location(duel, LOCATION_MZONE)
        && c.is_previous_position(duel, POS_FACEUP)
        && c.is_previous_set_card(duel, &[SET_FIENDSMITH])
        && c.is_reason(duel, REASON_EFFECT)
}

fn setcon(duel: &Duel, a: &Args) -> bool {
    let eg = a.eg.unwrap_or_default();
    a.rp == 1 - a.tp && !eg.is_contains(a.e.get_handler(duel)) && eg.is_exists(duel, |c, duel| setconfilter(c, duel, a.tp), 1, None)
}

fn settg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let c = a.e.get_handler(duel);
    if chk == 0 {
        return Ok(c.is_ssetable(duel));
    }
    duel.set_operation_info(0, CATEGORY_LEAVE_GRAVE, Some(c.into()), 1, a.tp, 0);
    Ok(false)
}

fn setop(duel: &mut Duel, a: &Args) -> Script<()> {
    let c = a.e.get_handler(duel);
    if c.is_relate_to_effect(duel, a.e) && c.is_ssetable(duel) {
        duel.sset(a.tp, c)?;
    }
    Ok(())
}
