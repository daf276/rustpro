// Mist Valley Soldier
use crate::prelude::*;
use crate::testing::{Check, MZONE, OPPONENT};

const ID: u32 = 22837504;

// Attacking Giant Soldier of Stone (DEF 2000), which survives the battle.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (ID, MZONE),
    (13039848, MZONE | OPPONENT),
    (13039848, MZONE | OPPONENT),
    (13039848, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_TOHAND);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_F);
    e1.set_code(duel, EVENT_DAMAGE_STEP_END);
    e1.set_condition_mut(duel, retcon);
    e1.set_target(duel, rettg);
    e1.set_operation(duel, retop);
    c.register_effect(duel, e1);
}

fn label_card(duel: &Duel, e: EffectId) -> Option<CardId> {
    e.get_label_object_group(duel).and_then(|g| g.get_first())
}

fn retcon(duel: &mut Duel, a: &Args) -> bool {
    if !a.e.get_handler(duel).is_relate_to_battle(duel) {
        return false;
    }
    let t = if a.ev == 0 { duel.get_attack_target() } else { duel.get_attacker() };
    a.e.set_label_object_group(duel, t.map(Into::into));
    t.is_some_and(|t| t.is_relate_to_battle(duel))
}

fn rettg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(true);
    }
    let t = label_card(duel, a.e);
    duel.set_operation_info(0, CATEGORY_TOHAND, t.map(Into::into), 1, 0, 0);
    Ok(false)
}

fn retop(duel: &mut Duel, a: &Args) -> Script<()> {
    if let Some(t) = label_card(duel, a.e)
        && t.is_relate_to_battle(duel)
    {
        duel.send_to_hand(t, PLAYER_NONE, REASON_EFFECT)?;
    }
    Ok(())
}
