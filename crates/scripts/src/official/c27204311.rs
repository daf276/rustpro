// Nibiru, the Primal Being
use crate::prelude::*;
use crate::testing::{Check, HAND, OPPONENT};

const ID: u32 = 27204311;

pub const LISTED_NAMES: &[u32] = &[27204312];
// A copy in each hand: the condition needs five summons by the opponent in one turn, which either side's combos reach.
pub const SUPPORT: &[(u32, u16)] = &[(ID, HAND), (ID, HAND | OPPONENT)];
pub const CHECKS: &[Check] = &[Check::Resolves, Check::SpecialSummonedCode(ID + 1)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_category(duel, CATEGORY_RELEASE + CATEGORY_SPECIAL_SUMMON + CATEGORY_TOKEN);
    e1.set_type(duel, EFFECT_TYPE_QUICK_O);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_range(duel, LOCATION_HAND);
    e1.set_hint_timing(duel, 0, TIMING_MAIN_END | TIMING_SUMMON | TIMING_SPSUMMON);
    e1.set_count_limit(duel, 1, ID);
    e1.set_condition(duel, condition);
    e1.set_target(duel, target);
    e1.set_operation(duel, operation);
    c.register_effect(duel, e1);
    aux::global_check(duel, ID, |duel| {
        let ge1 = duel.create_effect(c);
        ge1.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_CONTINUOUS);
        ge1.set_code(duel, EVENT_SPSUMMON_SUCCESS);
        ge1.set_operation(duel, checkop);
        duel.register_effect(ge1, 0);
        let ge2 = ge1.clone_effect(duel);
        ge2.set_code(duel, EVENT_SUMMON_SUCCESS);
        duel.register_effect(ge2, 0);
    });
}

fn checkop(duel: &mut Duel, a: &Args) -> Script<()> {
    for tc in a.eg.unwrap_or_default() {
        duel.register_flag_effect(tc.get_summon_player(duel), ID, RESET_PHASE | PHASE_END, 0, 1);
    }
    Ok(())
}

fn condition(duel: &Duel, a: &Args) -> bool {
    duel.get_flag_effect(1 - a.tp, ID) >= 5 && duel.is_main_phase()
}

fn releasable(duel: &Duel, tp: u8) -> CardSet {
    duel.get_matching_group(
        aux::faceup_filter(CardId::is_releasable_by_effect),
        tp,
        LOCATION_MZONE,
        LOCATION_MZONE,
        None,
    )
}

fn can_summon_token(duel: &mut Duel, tp: u8, atk: i64, def: i64) -> bool {
    let stats = MonsterStats::new(0, TYPES_TOKEN, atk as i32, def as i32, 11, RACE_ROCK, ATTRIBUTE_LIGHT);
    duel.is_player_can_special_summon_monster_full(tp, ID + 1, stats, POS_FACEUP, 1 - tp, 0)
}

fn target(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (c, tp) = (a.e.get_handler(duel), a.tp);
    let g = releasable(duel, tp);
    if chk == 0 {
        if g.is_empty()
            || duel.get_mzone_count_full(tp, Some(g), tp, LOCATION_REASON_TOFIELD, 0xff) <= 0
            || duel.get_mzone_count_full(1 - tp, Some(g), tp, LOCATION_REASON_TOFIELD, 0xff) <= 0
            || !duel.is_player_can_special_summon_count(tp, 2)
            || !c.is_can_be_special_summoned(duel, a.e, 0, tp, false, false)
        {
            return Ok(false);
        }
        let atk = g.get_sum(duel, |c, duel| c.get_base_attack(duel).into());
        let def = g.get_sum(duel, |c, duel| c.get_base_defense(duel).into());
        return Ok(can_summon_token(duel, tp, atk, def));
    }
    duel.set_operation_info(0, CATEGORY_RELEASE, Some(g), g.len() as u8, 0, 0);
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, Some(c.into()), 1, 0, 0);
    duel.set_operation_info(0, CATEGORY_TOKEN, None, 1, 0, 0);
    Ok(false)
}

fn operation(duel: &mut Duel, a: &Args) -> Script<()> {
    let (c, tp) = (a.e.get_handler(duel), a.tp);
    let g = releasable(duel, tp);
    if duel.release(g, REASON_EFFECT)? == 0 || !c.is_relate_to_effect(duel, a.e) {
        return Ok(());
    }
    let og = duel.get_operated_group();
    if duel.get_location_count(tp, LOCATION_MZONE) <= 0
        || duel.special_summon(c, 0, tp, tp, false, false, POS_FACEUP)? <= 0
        || duel.get_mzone_count_full(1 - tp, None, tp, LOCATION_REASON_TOFIELD, 0xff) <= 0
    {
        return Ok(());
    }
    let atk = og.get_sum(duel, |c, duel| c.get_base_attack(duel).max(0).into());
    let def = og.get_sum(duel, |c, duel| c.get_base_defense(duel).max(0).into());
    if !can_summon_token(duel, tp, atk, def) {
        return Ok(());
    }
    duel.break_effect()?;
    let Some(token) = duel.create_token(tp, ID + 1) else {
        return Ok(());
    };
    duel.special_summon_step(token, 0, tp, 1 - tp, false, false, POS_FACEUP)?;
    for (code, value) in [(EFFECT_SET_ATTACK, atk), (EFFECT_SET_DEFENSE, def)] {
        let e1 = duel.create_effect(c);
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_code(duel, code);
        e1.set_value(duel, value as i32);
        e1.set_reset(duel, RESET_EVENT | RESETS_STANDARD);
        token.register_effect(duel, e1);
    }
    duel.special_summon_complete()?;
    Ok(())
}
