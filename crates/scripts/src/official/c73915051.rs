// Scapegoat
use crate::prelude::*;
use crate::testing::Check;

const ID: u32 = 73915051;

pub const LISTED_NAMES: &[u32] = &[73915052];
pub const CHECKS: &[Check] = &[Check::Resolves, Check::SpecialSummonedCode(ID + 1)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON + CATEGORY_TOKEN);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_hint_timing(duel, 0, TIMING_END_PHASE);
    e1.set_cost(duel, cost);
    e1.set_target(duel, target);
    e1.set_operation(duel, activate);
    c.register_effect(duel, e1);
}

fn cost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.get_activity_count(tp, ACTIVITY_SUMMON) == 0
            && duel.get_activity_count(tp, ACTIVITY_FLIPSUMMON) == 0
            && duel.get_activity_count(tp, ACTIVITY_SPSUMMON) == 0);
    }
    let c = a.e.get_handler(duel);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_property(duel, EFFECT_FLAG_PLAYER_TARGET + EFFECT_FLAG_OATH);
    e1.set_code(duel, EFFECT_CANNOT_SPECIAL_SUMMON);
    e1.set_reset(duel, RESET_PHASE | PHASE_END);
    e1.set_target_range(duel, 1, 0);
    e1.set_label_object(duel, Some(a.e));
    e1.set_target_spsummon(duel, sumlimit);
    duel.register_effect(e1, tp);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_FIELD);
    e2.set_property(duel, EFFECT_FLAG_PLAYER_TARGET + EFFECT_FLAG_OATH);
    e2.set_code(duel, EFFECT_CANNOT_SUMMON);
    e2.set_reset(duel, RESET_PHASE | PHASE_END);
    e2.set_target_range(duel, 1, 0);
    duel.register_effect(e2, tp);
    let e3 = e2.clone_effect(duel);
    e3.set_code(duel, EFFECT_CANNOT_FLIP_SUMMON);
    duel.register_effect(e3, tp);
    let e4 = duel.create_effect(c);
    e4.set_property(duel, EFFECT_FLAG_PLAYER_TARGET + EFFECT_FLAG_CLIENT_HINT + EFFECT_FLAG_OATH);
    e4.set_description(duel, aux::stringid(ID, 1));
    e4.set_reset(duel, RESET_PHASE | PHASE_END);
    e4.set_target_range(duel, 1, 0);
    duel.register_effect(e4, tp);
    Ok(false)
}

fn sumlimit(duel: &Duel, e: EffectId, _: CardId, _: u8, _: u32, _: u8, _: u8, se: Option<EffectId>) -> bool {
    e.get_label_object(duel) != se
}

fn can_summon_tokens(duel: &mut Duel, tp: u8) -> bool {
    duel.is_player_affected_by_effect(tp, CARD_BLUEEYES_SPIRIT).is_none()
        && duel.get_location_count(tp, LOCATION_MZONE) > 3
        && duel.is_player_can_special_summon_monster(tp, ID + 1, MonsterStats::new(0, TYPES_TOKEN, 0, 0, 1, RACE_BEAST, ATTRIBUTE_EARTH))
}

fn target(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(can_summon_tokens(duel, a.tp));
    }
    duel.set_operation_info(0, CATEGORY_TOKEN, None, 4, 0, 0);
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 4, a.tp, 0);
    Ok(false)
}

fn activate(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    if !can_summon_tokens(duel, tp) {
        return Ok(());
    }
    for i in 1..=4 {
        let Some(token) = duel.create_token(tp, ID + i) else {
            return Ok(());
        };
        duel.special_summon_step(token, 0, tp, tp, false, false, POS_FACEUP_DEFENSE)?;
        let e1 = duel.create_effect(a.e.get_handler(duel));
        e1.set_description(duel, 3304);
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_code(duel, EFFECT_UNRELEASABLE_SUM);
        e1.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE + EFFECT_FLAG_CLIENT_HINT);
        e1.set_value(duel, 1);
        e1.set_reset(duel, RESET_EVENT | RESETS_STANDARD);
        token.register_effect_full(duel, e1, true);
    }
    duel.special_summon_complete()?;
    Ok(())
}
