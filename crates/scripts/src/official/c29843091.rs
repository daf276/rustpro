// Ojama Trio
use crate::prelude::*;
use crate::testing::{Check, HAND, SZONE};
use engine::constants::MSG_DAMAGE;

const ID: u32 = 29843091;

pub const LISTED_NAMES: &[u32] = &[TOKEN_OJAMA];
// Set Traps to activate, and Raigeki to destroy the Tokens for the damage. The Trap has no description, so Resolves would count Raigeki.
pub const SUPPORT: &[(u32, u16)] = &[(ID, SZONE), (ID, SZONE), (12580477, HAND)];
pub const CHECKS: &[Check] = &[Check::SpecialSummonedCode(TOKEN_OJAMA), Check::Message(MSG_DAMAGE)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON + CATEGORY_TOKEN);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_target(duel, target);
    e1.set_operation(duel, activate);
    e1.set_hint_timing(duel, 0, TIMING_END_PHASE);
    c.register_effect(duel, e1);
}

fn can_summon_token(duel: &mut Duel, tp: u8) -> bool {
    let stats = MonsterStats::new(SET_OJAMA, TYPES_TOKEN, 0, 1000, 2, RACE_BEAST, ATTRIBUTE_LIGHT);
    duel.is_player_can_special_summon_monster_full(tp, TOKEN_OJAMA, stats, POS_FACEUP_DEFENSE, 1 - tp, 0)
}

fn target(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.is_player_affected_by_effect(tp, CARD_BLUEEYES_SPIRIT).is_none()
            && duel.get_location_count_full(1 - tp, LOCATION_MZONE, tp, LOCATION_REASON_TOFIELD, 0xff) > 2
            && can_summon_token(duel, tp));
    }
    duel.set_operation_info(0, CATEGORY_TOKEN, None, 3, 0, 0);
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 3, tp, 0);
    Ok(false)
}

fn activate(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    if duel.is_player_affected_by_effect(tp, CARD_BLUEEYES_SPIRIT).is_some()
        || duel.get_location_count_full(1 - tp, LOCATION_MZONE, tp, LOCATION_REASON_TOFIELD, 0xff) < 3
        || !can_summon_token(duel, tp)
    {
        return Ok(());
    }
    for i in 0..3 {
        let Some(token) = duel.create_token(tp, TOKEN_OJAMA + i) else {
            continue;
        };
        if duel.special_summon_step(token, 0, tp, 1 - tp, false, false, POS_FACEUP_DEFENSE)? {
            let e1 = duel.create_effect(e.get_handler(duel));
            e1.set_description(duel, 3304);
            e1.set_type(duel, EFFECT_TYPE_SINGLE);
            e1.set_code(duel, EFFECT_UNRELEASABLE_SUM);
            e1.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE + EFFECT_FLAG_CLIENT_HINT);
            e1.set_reset(duel, RESET_EVENT | RESETS_STANDARD);
            e1.set_value(duel, 1);
            token.register_effect_full(duel, e1, true);
            let e2 = duel.create_effect(e.get_handler(duel));
            e2.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_CONTINUOUS);
            e2.set_code(duel, EVENT_LEAVE_FIELD);
            e2.set_operation(duel, damop);
            token.register_effect_full(duel, e2, true);
        }
    }
    duel.special_summon_complete()?;
    Ok(())
}

fn damop(duel: &mut Duel, a: &Args) -> Script<()> {
    let c = a.e.get_handler(duel);
    if c.is_reason(duel, REASON_DESTROY) {
        duel.damage(c.get_previous_controler(duel), 300, REASON_EFFECT)?;
    }
    a.e.reset(duel);
    Ok(())
}
