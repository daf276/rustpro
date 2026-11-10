// The Eye of Truth
use crate::prelude::*;
use crate::testing::{Check, HAND, OPPONENT};

const ID: u32 = 34694160;

// In the hand, with Spells (Dark Hole) in the opponent's hand for the recovery.
pub const SUPPORT: &[(u32, u16)] = &[(ID, HAND), (53129443, HAND | OPPONENT), (53129443, HAND | OPPONENT)];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_FIELD);
    e2.set_code(duel, EFFECT_PUBLIC);
    e2.set_range(duel, LOCATION_SZONE);
    e2.set_target_range(duel, 0, LOCATION_HAND);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 0));
    e3.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_F);
    e3.set_category(duel, CATEGORY_RECOVER);
    e3.set_property(duel, EFFECT_FLAG_PLAYER_TARGET);
    e3.set_code(duel, EVENT_PHASE | PHASE_STANDBY);
    e3.set_range(duel, LOCATION_SZONE);
    e3.set_count_limit(duel, 1, 0);
    e3.set_condition(duel, reccon);
    e3.set_target(duel, rectg);
    e3.set_operation(duel, recop);
    c.register_effect(duel, e3);
}

fn reccon(duel: &Duel, a: &Args) -> bool {
    let tp = a.tp;
    duel.is_turn_player(1 - tp) && duel.is_existing_matching_card(CardId::is_spell, tp, 0, LOCATION_HAND, 1, None)
}

fn rectg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(true);
    }
    duel.set_target_player(1 - tp);
    duel.set_target_param(1000);
    duel.set_operation_info(0, CATEGORY_RECOVER, None, 0, 1 - tp, 1000);
    Ok(false)
}

fn recop(duel: &mut Duel, a: &Args) -> Script<()> {
    if !a.e.get_handler(duel).is_relate_to_effect(duel, a.e) {
        return Ok(());
    }
    let p: u8 = duel.get_chain_info(0, CHAININFO_TARGET_PLAYER);
    let d: u32 = duel.get_chain_info(0, CHAININFO_TARGET_PARAM);
    duel.recover(p, d, REASON_EFFECT)?;
    Ok(())
}
