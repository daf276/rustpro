// Argostars - Lightning Tydeu
use crate::prelude::*;
use crate::testing::{Check, DECK, HAND, MZONE, SZONE};

const ID: u32 = 65889305;

pub const LISTED_SERIES: &[u16] = &[SET_ARGOSTARS];
pub const LISTED_NAMES: &[u32] = &[ID];
// Set copies to activate, Argostars cards to add, and Warriors to boost.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, SZONE),
    (ID, SZONE),
    (ID, HAND),
    (91438674, DECK),
    (91438674, DECK),
    (91284003, DECK),
    (91438674, MZONE),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(0),
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e0 = duel.create_effect(c);
    e0.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e0.set_property(duel, EFFECT_FLAG_DAMAGE_STEP);
    e0.set_code(duel, EVENT_FREE_CHAIN);
    e0.set_condition(duel, aux::stat_change_damage_step_condition);
    e0.set_hint_timing(duel, TIMING_DAMAGE_STEP, TIMING_DAMAGE_STEP);
    c.register_effect(duel, e0);
    let e1a = duel.create_effect(c);
    e1a.set_type(duel, EFFECT_TYPE_FIELD);
    e1a.set_code(duel, EFFECT_UPDATE_ATTACK);
    e1a.set_range(duel, LOCATION_SZONE);
    e1a.set_target_range(duel, LOCATION_MZONE, 0);
    e1a.set_target_card(duel, |duel, _, c| c.is_race(duel, RACE_WARRIOR));
    e1a.set_value(duel, 500);
    c.register_effect(duel, e1a);
    let e1b = e1a.clone_effect(duel);
    e1b.set_code(duel, EFFECT_UPDATE_DEFENSE);
    c.register_effect(duel, e1b);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 0));
    e2.set_category(duel, CATEGORY_SPECIAL_SUMMON + CATEGORY_SEARCH + CATEGORY_TOHAND);
    e2.set_type(duel, EFFECT_TYPE_QUICK_O);
    e2.set_code(duel, EVENT_FREE_CHAIN);
    e2.set_range(duel, LOCATION_SZONE);
    e2.set_count_limit(duel, 1, 0);
    e2.set_cost(duel, Cost::discard);
    e2.set_target(duel, sptg);
    e2.set_operation(duel, spop);
    e2.set_hint_timing(duel, 0, TIMING_STANDBY_PHASE | TIMING_MAIN_END | TIMINGS_CHECK_MONSTER_E);
    c.register_effect(duel, e2);
}

fn can_summon<const ID: u32, const ATK: i32>(duel: &mut Duel, tp: u8) -> bool {
    let stats = MonsterStats::new(
        SET_ARGOSTARS,
        TYPE_MONSTER | TYPE_EFFECT,
        ATK,
        ATK,
        4,
        RACE_WARRIOR,
        ATTRIBUTE_LIGHT,
    );
    duel.get_location_count(tp, LOCATION_MZONE) > 0 && duel.is_player_can_special_summon_monster(tp, ID, stats)
}

// sptg of the Argostars Continuous Traps with ATK and DEF `ATK`; `possible` is the category, count, player and parameter of its possible operation.
pub fn trap_sptg<const ID: u32, const ATK: i32>(duel: &mut Duel, a: &Args, chk: i32, possible: (u64, u8, u8, i32)) -> Script<bool> {
    let (c, tp) = (a.e.get_handler(duel), a.tp);
    if chk == 0 {
        return Ok(!c.has_flag_effect(duel, ID) && can_summon::<ID, ATK>(duel, tp));
    }
    c.register_flag_effect(
        duel,
        ID,
        RESETS_STANDARD_PHASE_END & !(RESET_TOFIELD | RESET_LEAVE | RESET_TURN_SET),
        0,
        1,
    );
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, Some(c.into()), 1, tp, 0);
    let (category, count, player, param) = possible;
    duel.set_possible_operation_info(0, category, None, count, player, param);
    Ok(false)
}

// The summon spop of the Argostars Continuous Traps starts with: SpecialSummonComplete's count, or None where the Lua returns first.
pub fn trap_summon<const ID: u32, const ATK: i32>(duel: &mut Duel, a: &Args) -> Script<Option<i32>> {
    let (c, tp) = (a.e.get_handler(duel), a.tp);
    if !c.is_relate_to_effect(duel, a.e) || !can_summon::<ID, ATK>(duel, tp) {
        return Ok(None);
    }
    c.add_monster_attribute(duel, TYPE_EFFECT | TYPE_TRAP);
    duel.special_summon_step(c, 0, tp, tp, true, false, POS_FACEUP)?;
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 1));
    e1.set_type(duel, EFFECT_TYPE_QUICK_O);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_count_limit(duel, 1, 0);
    e1.set_target(duel, |duel, a, chk, _| {
        Ok(chk == 0 && duel.get_location_count(a.tp, LOCATION_SZONE) > 0)
    });
    e1.set_operation(duel, |duel, a| {
        let c = a.e.get_handler(duel);
        if c.is_relate_to_effect(duel, a.e) {
            duel.move_to_field(c, a.tp, a.tp, LOCATION_SZONE, POS_FACEUP, true)?;
        }
        Ok(())
    });
    e1.set_hint_timing(duel, 0, TIMING_STANDBY_PHASE | TIMING_MAIN_END | TIMINGS_CHECK_MONSTER_E);
    e1.set_reset(duel, RESET_EVENT | RESETS_STANDARD);
    c.register_effect(duel, e1);
    c.add_monster_attribute_complete(duel);
    Ok(Some(duel.special_summon_complete()?))
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    trap_sptg::<ID, 2000>(duel, a, chk, (CATEGORY_TOHAND, 1, a.tp, LOCATION_DECK as i32))
}

fn thfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_ARGOSTARS]) && !c.is_code(duel, ID) && c.is_able_to_hand(duel)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    if trap_summon::<ID, 2000>(duel, a)?.is_none_or(|count| count == 0) {
        return Ok(());
    }
    let g = duel.get_matching_group(thfilter, tp, LOCATION_DECK, 0, None);
    if !g.is_empty() && duel.select_yes_no(tp, aux::stringid(ID, 2))? {
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_ATOHAND);
        let sg = g.select(duel, tp, 1, 1, None)?;
        if !sg.is_empty() {
            duel.break_effect()?;
            duel.send_to_hand(sg, PLAYER_NONE, REASON_EFFECT)?;
            duel.confirm_cards(1 - tp, sg)?;
        }
    }
    Ok(())
}
