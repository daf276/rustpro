// The Black Goat Laughs
use crate::prelude::*;
use crate::testing::{Check, GRAVE, SZONE};

const ID: u32 = 49299410;

// Set on the field to activate, and in the Graveyard to banish.
pub const SUPPORT: &[(u32, u16)] = &[(ID, SZONE), (ID, GRAVE)];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_hint_timing(duel, 0, TIMING_STANDBY_PHASE | TIMINGS_CHECK_MONSTER);
    e1.set_count_limit(duel, 1, ID);
    e1.set_target(duel, decltg);
    e1.set_operation(duel, sumlimop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_type(duel, EFFECT_TYPE_QUICK_O);
    e2.set_code(duel, EVENT_FREE_CHAIN);
    e2.set_range(duel, LOCATION_GRAVE);
    e2.set_hint_timing(duel, 0, TIMING_STANDBY_PHASE | TIMINGS_CHECK_MONSTER);
    e2.set_count_limit(duel, 1, ID);
    e2.set_cost(duel, Cost::self_banish);
    e2.set_target(duel, decltg);
    e2.set_operation(duel, efflimop);
    c.register_effect(duel, e2);
}

fn decltg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(true);
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_CODE);
    let code = duel.announce_card(tp, &[u64::from(TYPE_MONSTER), OPCODE_ISTYPE])?;
    duel.set_target_param(code);
    duel.set_operation_info(0, CATEGORY_ANNOUNCE, None, 0, tp, ANNOUNCE_CARD_FILTER as i32);
    Ok(false)
}

fn sumlimop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    let code: u32 = duel.get_chain_info(0, CHAININFO_TARGET_PARAM);
    let e1 = duel.create_effect(a.e.get_handler(duel));
    e1.set_description(duel, aux::stringid(ID, 2));
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_property(duel, EFFECT_FLAG_PLAYER_TARGET + EFFECT_FLAG_CLIENT_HINT);
    e1.set_code(duel, EFFECT_CANNOT_SPECIAL_SUMMON);
    e1.set_target_range(duel, 1, 1);
    e1.set_label(duel, i64::from(code));
    e1.set_target_spsummon(duel, |duel, e, c, _, _, _, _, _| {
        c.is_original_code_rule(duel, e.get_label(duel) as u32) && !c.is_location(duel, LOCATION_GRAVE)
    });
    e1.set_reset(duel, RESET_PHASE | PHASE_END);
    duel.register_effect(e1, tp);
    Ok(())
}

fn efflimop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    let code: u32 = duel.get_chain_info(0, CHAININFO_TARGET_PARAM);
    let e1 = duel.create_effect(a.e.get_handler(duel));
    e1.set_description(duel, aux::stringid(ID, 3));
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_property(duel, EFFECT_FLAG_PLAYER_TARGET + EFFECT_FLAG_CLIENT_HINT);
    e1.set_code(duel, EFFECT_CANNOT_ACTIVATE);
    e1.set_target_range(duel, 1, 1);
    e1.set_label(duel, i64::from(code));
    e1.set_value(
        duel,
        Value::EffectPlayer(|duel, e, re, _| {
            re.get_handler(duel).is_original_code_rule(duel, e.get_label(duel) as u32)
                && re.is_monster_effect(duel)
                && re.get_activate_location(duel) == LOCATION_MZONE
        }),
    );
    e1.set_reset(duel, RESET_PHASE | PHASE_END);
    duel.register_effect(e1, tp);
    Ok(())
}
