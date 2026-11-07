// Sky Striker Mecha - Hornet Drones
use crate::prelude::*;
use crate::testing::{Check, GRAVE, HAND};

const ID: u32 = 52340444;

pub const LISTED_NAMES: &[u32] = &[ID + 1];
// In the hand with three Spells in the GY for the stronger Token.
pub const SUPPORT: &[(u32, u16)] = &[(ID, HAND), (98338152, GRAVE), (98338152, GRAVE), (98338152, GRAVE)];
pub const CHECKS: &[Check] = &[Check::Resolves, Check::SpecialSummonedCode(ID + 1)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON + CATEGORY_TOKEN);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_hint_timing(duel, 0, TIMING_END_PHASE);
    e1.set_condition(duel, condition);
    e1.set_target(duel, target);
    e1.set_operation(duel, activate);
    c.register_effect(duel, e1);
}

fn condition(duel: &Duel, a: &Args) -> bool {
    duel.get_field_group_count(a.tp, LOCATION_MMZONE, 0) == 0
}

fn three_spells(duel: &Duel, tp: u8) -> bool {
    duel.get_matching_group_count(CardId::is_spell, tp, LOCATION_GRAVE, 0, None) >= 3
}

fn can_summon_token(duel: &mut Duel, tp: u8) -> bool {
    let atk = if three_spells(duel, tp) { 1500 } else { 0 };
    let stats = MonsterStats::new(SET_SKY_STRIKER_ACE, TYPES_TOKEN, atk, atk, 1, RACE_WARRIOR, ATTRIBUTE_DARK);
    duel.is_player_can_special_summon_monster_full(tp, ID + 1, stats, POS_FACEUP_DEFENSE, tp, 0)
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
    let (c, tp) = (a.e.get_handler(duel), a.tp);
    if duel.get_location_count(tp, LOCATION_MZONE) <= 0 || !can_summon_token(duel, tp) {
        return Ok(());
    }
    let Some(token) = duel.create_token(tp, ID + 1) else {
        return Ok(());
    };
    if duel.special_summon_step(token, 0, tp, tp, false, false, POS_FACEUP_DEFENSE)? {
        let e1 = duel.create_effect(c);
        e1.set_description(duel, 3303);
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_code(duel, EFFECT_UNRELEASABLE_SUM);
        e1.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE + EFFECT_FLAG_CLIENT_HINT);
        e1.set_value(duel, 1);
        e1.set_reset(duel, RESET_EVENT | RESETS_STANDARD);
        token.register_effect(duel, e1);
        let e2 = e1.clone_effect(duel);
        e2.set_code(duel, EFFECT_UNRELEASABLE_NONSUM);
        token.register_effect(duel, e2);
        if three_spells(duel, tp) {
            for code in [EFFECT_SET_ATTACK, EFFECT_SET_DEFENSE] {
                let e3 = duel.create_effect(c);
                e3.set_type(duel, EFFECT_TYPE_SINGLE);
                e3.set_code(duel, code);
                e3.set_value(duel, 1500);
                e3.set_reset(duel, RESET_EVENT | RESETS_STANDARD);
                token.register_effect(duel, e3);
            }
        }
    }
    duel.special_summon_complete()?;
    Ok(())
}
