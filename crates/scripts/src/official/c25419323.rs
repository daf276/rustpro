// Dinomorphia Shell
use crate::prelude::*;
use crate::testing::{Check, GRAVE, HAND, SZONE};

const ID: u32 = 25419323;

pub const LISTED_NAMES: &[u32] = &[25419324];
// Set to be activated at the start of the opponent's Battle Phase, and a copy in the GY.
pub const SUPPORT: &[(u32, u16)] = &[(ID, SZONE), (ID, SZONE), (ID, HAND), (ID, GRAVE)];
pub const CHECKS: &[Check] = &[Check::SpecialSummonedCode(ID + 1), Check::ResolvesEffect(aux::stringid(ID, 1))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON + CATEGORY_TOKEN);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_count_limit_full(duel, 1, ID, EFFECT_COUNT_CODE_OATH);
    e1.set_hint_timing(duel, TIMING_BATTLE_START, TIMING_BATTLE_START);
    e1.set_condition(duel, |duel, a| duel.is_phase(PHASE_BATTLE_START) && duel.is_turn_player(1 - a.tp));
    e1.set_cost(duel, super::c7336745::cost);
    e1.set_target(duel, target);
    e1.set_operation(duel, activate);
    c.register_effect(duel, e1);
    super::c7336745::register_no_battle_damage(duel, c, ID);
}

fn can_summon_token(duel: &mut Duel, tp: u8) -> bool {
    let stats = MonsterStats::new(0, TYPES_TOKEN, 0, 3000, 10, RACE_DINOSAUR, ATTRIBUTE_DARK);
    duel.is_player_can_special_summon_monster_full(tp, ID + 1, stats, POS_FACEUP, tp, 0)
}

fn target(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.get_location_count(tp, LOCATION_MZONE) > 0 && can_summon_token(duel, tp));
    }
    duel.set_operation_info(0, CATEGORY_TOKEN, None, 1, 0, 0);
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, tp, 0);
    Ok(false)
}

fn activate(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    if duel.get_location_count(tp, LOCATION_MZONE) == 0 || !can_summon_token(duel, tp) {
        return Ok(());
    }
    let Some(token) = duel.create_token(tp, ID + 1) else {
        return Ok(());
    };
    if duel.special_summon(token, 0, tp, tp, false, false, POS_FACEUP)? > 0 {
        let e1 = duel.create_effect(a.e.get_handler(duel));
        e1.set_type(duel, EFFECT_TYPE_FIELD);
        e1.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE);
        e1.set_code(duel, EFFECT_CANNOT_SELECT_BATTLE_TARGET);
        e1.set_range(duel, LOCATION_MZONE);
        e1.set_target_range(duel, 0, LOCATION_MZONE);
        e1.set_reset(duel, RESETS_STANDARD_PHASE_END);
        e1.set_value(duel, Value::Card(|duel, e, c| i32::from(c != e.get_handler(duel))));
        token.register_effect(duel, e1);
    }
    Ok(())
}
