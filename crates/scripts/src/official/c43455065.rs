// Magical Spring
use crate::prelude::*;
use crate::testing::{HAND, OPPONENT, SZONE};

const ID: u32 = 43455065;

// In the hand, with Skill Drain and Toon World (Continuous cards that can sit face-up) on both sides.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (82732705, SZONE | OPPONENT),
    (15259703, SZONE | OPPONENT),
    (82732705, SZONE),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_category(duel, CATEGORY_DRAW);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_property(duel, EFFECT_FLAG_PLAYER_TARGET);
    e1.set_count_limit_full(duel, 1, ID, EFFECT_COUNT_CODE_OATH);
    e1.set_target(duel, target);
    e1.set_operation(duel, activate);
    c.register_effect(duel, e1);
}

fn filter(c: CardId, duel: &Duel) -> bool {
    c.is_faceup(duel) && c.is_spell_trap(duel)
}

fn target(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    let ct = duel.get_matching_group_count(filter, tp, 0, LOCATION_ONFIELD, None);
    if chk == 0 {
        return Ok(ct > 0 && duel.is_player_can_draw(tp, ct));
    }
    duel.set_target_player(tp);
    duel.set_target_param(ct as u32);
    duel.set_operation_info(0, CATEGORY_DRAW, None, 0, tp, ct as i32);
    Ok(false)
}

fn activate(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let p: u8 = duel.get_chain_info(0, CHAININFO_TARGET_PLAYER);
    let ct = duel.get_matching_group_count(filter, tp, 0, LOCATION_ONFIELD, None);
    if duel.draw(p, ct as u32, REASON_EFFECT)? != 0 {
        let ct = duel.get_matching_group_count(filter, tp, LOCATION_ONFIELD, 0, None);
        if ct > 0 {
            duel.break_effect()?;
            duel.discard_hand(tp, aux::TRUE, ct as u8, ct as u8, REASON_EFFECT | REASON_DISCARD)?;
        }
    }
    let c = e.get_handler(duel);
    let rct = if duel.is_turn_player(1 - tp) { 2 } else { 1 };
    let reset = RESET_PHASE | PHASE_END | RESET_OPPO_TURN;
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_code(duel, EFFECT_INDESTRUCTABLE_EFFECT);
    e1.set_target_range(duel, 0, LOCATION_ONFIELD);
    e1.set_property(duel, EFFECT_FLAG_SET_AVAILABLE);
    e1.set_target_card(duel, |duel, _, c| c.is_spell_trap(duel));
    e1.set_value(duel, 1);
    e1.set_reset_full(duel, reset, rct);
    let e2 = e1.clone_effect(duel);
    e2.set_code(duel, EFFECT_INDESTRUCTABLE_BATTLE);
    e2.set_target_range(duel, 0, LOCATION_MZONE);
    duel.register_effect(e1, tp);
    duel.register_effect(e2, tp);
    for code in [EFFECT_CANNOT_INACTIVATE, EFFECT_CANNOT_DISEFFECT] {
        let e = duel.create_effect(c);
        e.set_type(duel, EFFECT_TYPE_FIELD);
        e.set_code(duel, code);
        e.set_value(duel, Value::Chain(efilter));
        e.set_reset_full(duel, reset, rct);
        duel.register_effect(e, tp);
    }
    let e4 = duel.create_effect(c);
    e4.set_type(duel, EFFECT_TYPE_FIELD);
    e4.set_code(duel, EFFECT_CANNOT_DISABLE);
    e4.set_target_range(duel, 0, LOCATION_ONFIELD);
    e4.set_property(duel, EFFECT_FLAG_SET_AVAILABLE);
    e4.set_target_card(duel, |duel, _, c| c.is_spell_trap(duel));
    e4.set_reset_full(duel, reset, rct);
    duel.register_effect(e4, tp);
    Ok(())
}

fn efilter(duel: &Duel, e: EffectId, ct: u8) -> bool {
    let Some(te) = duel.get_chain_info::<Option<EffectId>>(ct, CHAININFO_TRIGGERING_EFFECT) else {
        return false;
    };
    let tp: u8 = duel.get_chain_info(ct, CHAININFO_TRIGGERING_PLAYER);
    tp != e.get_handler_player(duel) && te.get_handler(duel).is_spell_trap(duel)
}
