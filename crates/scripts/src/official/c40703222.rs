// Multiply
use crate::prelude::*;
use crate::testing::{Check, HAND, MZONE};

const ID: u32 = 40703222;

pub const LISTED_NAMES: &[u32] = &[CARD_KURIBOH, TOKEN_KURIBOH];
// In the hand, with Kuriboh on the field to Tribute.
pub const SUPPORT: &[(u32, u16)] = &[(ID, HAND), (ID, HAND), (CARD_KURIBOH, MZONE), (CARD_KURIBOH, MZONE)];
pub const CHECKS: &[Check] = &[Check::SpecialSummonedCode(TOKEN_KURIBOH)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON + CATEGORY_TOKEN);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_cost(duel, cost);
    e1.set_target(duel, target);
    e1.set_operation(duel, activate);
    e1.set_hint_timing(duel, 0, TIMING_END_PHASE);
    c.register_effect(duel, e1);
}

fn token_stats() -> MonsterStats {
    MonsterStats::new(0, TYPES_TOKEN, 300, 200, 1, RACE_FIEND, ATTRIBUTE_DARK)
}

fn cfilter(c: CardId, duel: &Duel, ft: i32, tp: u8) -> bool {
    c.is_faceup(duel)
        && c.is_code(duel, CARD_KURIBOH)
        && (ft > 0 || (c.get_sequence(duel) < 5 && c.is_controler(duel, tp)))
        && (c.is_faceup(duel) || c.is_controler(duel, tp))
}

fn cost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let tp = a.tp;
    a.e.set_label(duel, 1);
    let ft = duel.get_location_count(tp, LOCATION_MZONE);
    let filter = |c, duel: &Duel| cfilter(c, duel, ft, tp);
    if chk == 0 {
        return Ok(ft > -1 && duel.check_release_group_cost(tp, filter, (1, 1), false, None, None));
    }
    let g = duel.select_release_group_cost(tp, filter, (1, 1), false, None, None)?;
    duel.release(g, REASON_COST)?;
    Ok(false)
}

fn target(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if chk == 0 {
        if e.get_label(duel) == 0 && duel.get_location_count(tp, LOCATION_MZONE) <= 0 {
            return Ok(false);
        }
        e.set_label(duel, 0);
        return Ok(duel.is_player_can_special_summon_monster(tp, TOKEN_KURIBOH, token_stats()));
    }
    let mut ft = duel.get_location_count(tp, LOCATION_MZONE);
    if duel.is_player_affected_by_effect(tp, CARD_BLUEEYES_SPIRIT).is_some() {
        ft = 1;
    }
    duel.set_operation_info(0, CATEGORY_TOKEN, None, ft as u8, tp, 0);
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, ft as u8, tp, 0);
    Ok(false)
}

fn activate(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let mut ft = duel.get_location_count(tp, LOCATION_MZONE);
    if ft <= 0 || !duel.is_player_can_special_summon_monster(tp, TOKEN_KURIBOH, token_stats()) {
        return Ok(());
    }
    if duel.is_player_affected_by_effect(tp, CARD_BLUEEYES_SPIRIT).is_some() {
        ft = 1;
    }
    for _ in 0..ft {
        if let Some(token) = duel.create_token(tp, TOKEN_KURIBOH) {
            duel.special_summon_step(token, 0, tp, tp, false, false, POS_FACEUP_DEFENSE)?;
            let e1 = duel.create_effect(e.get_handler(duel));
            e1.set_description(duel, 3304);
            e1.set_type(duel, EFFECT_TYPE_SINGLE);
            e1.set_code(duel, EFFECT_UNRELEASABLE_SUM);
            e1.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE + EFFECT_FLAG_CLIENT_HINT);
            e1.set_value(duel, 1);
            e1.set_reset(duel, RESET_EVENT | RESETS_STANDARD);
            token.register_effect_full(duel, e1, true);
        }
    }
    duel.special_summon_complete()?;
    Ok(())
}
