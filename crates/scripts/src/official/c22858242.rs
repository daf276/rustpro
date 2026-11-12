// Zeman the Ape King
use crate::prelude::*;
use crate::testing::{Check, EXTRA, MZONE};

const ID: u32 = 22858242;

pub struct Procedure;

impl SynchroProcedure for Procedure {
    const MIN2: usize = 1;
    const MAX2: usize = 1;

    fn f1(c: CardId, duel: &Duel, sc: CardId, sumtype: u64, tp: u8) -> bool {
        c.is_attribute_full(duel, ATTRIBUTE_DARK, Some(sc), sumtype, tp)
    }

    fn f2(c: CardId, duel: &Duel, sc: CardId, sumtype: u64, tp: u8) -> bool {
        Synchro::non_tuner(c, duel, sc, tp, |c, duel| c.is_race_full(duel, RACE_BEAST, Some(sc), sumtype, tp))
    }
}

pub const SUPPORT: &[(u32, u16)] = &[(ID, MZONE), (ID, MZONE), (ID, EXTRA)];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    Synchro::add_procedure::<Procedure>(duel, c);
    c.enable_revive_limit(duel);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_property(duel, EFFECT_FLAG_PLAYER_TARGET);
    e1.set_code(duel, EFFECT_CANNOT_ACTIVATE);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_target_range(duel, 0, 1);
    e1.set_value(duel, Value::EffectPlayer(aclimit));
    e1.set_condition(duel, actcon);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 0));
    e2.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e2.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e2.set_code(duel, EVENT_ATTACK_ANNOUNCE);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_condition(duel, condition);
    e2.set_cost(duel, cost);
    e2.set_target(duel, target);
    e2.set_operation(duel, activate);
    c.register_effect(duel, e2);
}

pub fn aclimit(duel: &Duel, _: EffectId, re: EffectId, _: u8) -> bool {
    re.is_has_type(duel, EFFECT_TYPE_ACTIVATE)
}

pub fn actcon(duel: &Duel, a: &Args) -> bool {
    duel.get_attacker() == Some(a.e.get_handler(duel))
}

fn condition(duel: &Duel, a: &Args) -> bool {
    a.eg.and_then(|eg| eg.get_first()).is_some_and(|tc| tc.is_controler(duel, 1 - a.tp))
}

fn cfilter(c: CardId, duel: &Duel) -> bool {
    c.is_monster(duel) && c.is_able_to_grave_as_cost(duel)
}

fn cost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    stock::send_to_grave_cost(duel, a, chk, cfilter, LOCATION_MZONE | LOCATION_HAND)
}

fn target(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tg = duel.get_attacker();
    if let Some(chkc) = chkc {
        return Ok(Some(chkc) == tg);
    }
    // The Lua errors on a nil attacker.
    let Some(tg) = tg else {
        unported!(duel, true);
        return Ok(false);
    };
    if chk == 0 {
        return Ok(tg.is_on_field(duel) && tg.is_can_be_effect_target(duel, a.e));
    }
    duel.set_target_card(tg);
    Ok(false)
}

fn activate(duel: &mut Duel, _: &Args) -> Script<()> {
    duel.negate_attack()?;
    Ok(())
}
