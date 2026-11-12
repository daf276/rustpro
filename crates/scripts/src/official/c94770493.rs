// Double or Nothing!
use crate::prelude::*;
use crate::testing::{Check, HAND, MZONE, OPPONENT, SZONE};

const ID: u32 = 94770493;

// In the hand with Gagaga-like attackers on the field, against the opponent's Negate Attack and Sangan to attack into.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, HAND),
    (5818798, MZONE),
    (5818798, MZONE),
    (14315573, SZONE | OPPONENT),
    (14315573, SZONE | OPPONENT),
    (26202165, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(0)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_category(duel, CATEGORY_ATKCHANGE);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_ATTACK_DISABLED);
    e1.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e1.set_target(duel, target);
    e1.set_operation(duel, operation);
    c.register_effect(duel, e1);
}

fn target(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let e = a.e;
    let eg = a.eg.unwrap_or_default().get_first();
    if let Some(chkc) = chkc {
        return Ok(Some(chkc) == eg);
    }
    let Some(tc) = eg else { return Ok(false) };
    if chk == 0 {
        return Ok(tc.is_faceup(duel) && tc.is_can_be_effect_target(duel, e));
    }
    duel.set_target_card(tc);
    Ok(false)
}

fn operation(duel: &mut Duel, a: &Args) -> Script<()> {
    let e = a.e;
    let c = e.get_handler(duel);
    let Some(tc) = duel.get_first_target() else { return Ok(()) };
    if tc.is_faceup(duel) && tc.is_relate_to_effect(duel, e) && tc.get_flag_effect(duel, ID) == 0 {
        tc.register_flag_effect(duel, ID, RESETS_STANDARD_PHASE_END, 0, 1);
        let e1 = duel.create_effect(c);
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_code(duel, EFFECT_EXTRA_ATTACK);
        e1.set_value(duel, tc.get_attack_announced_count(duel) as i32);
        e1.set_reset(duel, RESETS_STANDARD_PHASE_END);
        tc.register_effect(duel, e1);
        let e2 = duel.create_effect(c);
        e2.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_CONTINUOUS);
        e2.set_code(duel, EVENT_BATTLE_START);
        e2.set_count_limit(duel, 1, 0);
        e2.set_operation(duel, atkop);
        e2.set_reset(duel, RESETS_STANDARD_PHASE_END);
        tc.register_effect(duel, e2);
    }
    Ok(())
}

fn atkop(duel: &mut Duel, a: &Args) -> Script<()> {
    let e = a.e;
    let c = e.get_handler(duel);
    let e1 = duel.create_effect(e.get_owner(duel));
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_property(duel, EFFECT_FLAG_SINGLE_RANGE);
    e1.set_code(duel, EFFECT_SET_ATTACK_FINAL);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_value(duel, c.get_attack(duel) * 2);
    e1.set_reset(duel, RESET_EVENT | RESETS_STANDARD | RESET_PHASE | PHASE_DAMAGE);
    c.register_effect(duel, e1);
    Ok(())
}
