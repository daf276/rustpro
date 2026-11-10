// Reflect Bounder
use crate::prelude::*;
use crate::testing::{Check, MZONE, OPPONENT};

const ID: u32 = 2851070;

// Two on the field in Attack Position, and the opponent's strong attackers.
pub const SUPPORT: &[(u32, u16)] = &[(ID, MZONE), (ID, MZONE), (13597785, MZONE | OPPONENT), (13597785, MZONE | OPPONENT)];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_DAMAGE);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_F);
    e1.set_code(duel, EVENT_BATTLE_CONFIRM);
    e1.set_condition(duel, damcon);
    e1.set_target(duel, damtg);
    e1.set_operation(duel, damop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_DESTROY);
    e2.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_F);
    e2.set_code(duel, EVENT_BATTLED);
    e2.set_target(duel, destg);
    e2.set_operation(duel, desop);
    c.register_effect(duel, e2);
}

fn damcon(duel: &Duel, a: &Args) -> bool {
    let c = duel.get_attack_target();
    c == Some(a.e.get_handler(duel)) && c.is_some_and(|c| c.get_battle_position(duel) == POS_FACEUP_ATTACK)
}

fn damtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(true);
    }
    duel.set_target_player(1 - a.tp);
    duel.set_operation_info(0, CATEGORY_DAMAGE, None, 0, 1 - a.tp, 0);
    Ok(false)
}

fn damop(duel: &mut Duel, a: &Args) -> Script<()> {
    let p: u8 = duel.get_chain_info(0, CHAININFO_TARGET_PLAYER);
    let lp1 = duel.get_lp(p);
    let atk = duel.get_attacker().map_or(0, |c| c.get_attack(duel));
    duel.damage(p, atk as u32, REASON_EFFECT)?;
    if duel.get_lp(p) < lp1 {
        a.e.get_handler(duel)
            .register_flag_effect(duel, ID, RESET_EVENT | RESETS_STANDARD | RESET_PHASE | PHASE_DAMAGE, 0, 1);
    }
    Ok(())
}

fn destg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let c = a.e.get_handler(duel);
    if chk == 0 {
        return Ok(c.get_flag_effect(duel, ID) != 0);
    }
    duel.set_operation_info(0, CATEGORY_DESTROY, Some(c.into()), 1, 0, 0);
    Ok(false)
}

fn desop(duel: &mut Duel, a: &Args) -> Script<()> {
    let c = a.e.get_handler(duel);
    if c.is_relate_to_effect(duel, a.e) {
        duel.destroy(c, REASON_EFFECT)?;
    }
    Ok(())
}
