// Garmr of the Nordic Beasts
use crate::prelude::*;
use crate::testing::{Check, MZONE, OPPONENT};

const ID: u32 = 88283496;

// Attacking Giant Soldier of Stone (Level 3, DEF 2000), which survives the battle.
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
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e1.set_code(duel, EVENT_BATTLED);
    e1.set_condition_mut(duel, retcon);
    e1.set_target(duel, rettg);
    e1.set_operation(duel, retop);
    c.register_effect(duel, e1);
}

fn label_card(duel: &Duel, e: EffectId) -> Option<CardId> {
    e.get_label_object_group(duel).and_then(|g| g.get_first())
}

fn retcon(duel: &mut Duel, a: &Args) -> bool {
    let mut tc = duel.get_attacker();
    if tc == Some(a.e.get_handler(duel)) {
        tc = duel.get_attack_target();
    }
    let Some(tc) = tc else { return false };
    a.e.set_label_object_group(duel, Some(tc.into()));
    let lv = tc.get_level(duel);
    lv > 0 && lv <= 4 && !tc.is_status(duel, STATUS_BATTLE_DESTROYED)
}

fn rettg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tc = label_card(duel, a.e);
    if chk == 0 {
        return Ok(tc.is_some_and(|tc| tc.is_able_to_hand(duel)));
    }
    duel.set_operation_info(0, CATEGORY_TOHAND, tc.map(Into::into), 0, 0, 0);
    Ok(false)
}

fn retop(duel: &mut Duel, a: &Args) -> Script<()> {
    if let Some(tc) = label_card(duel, a.e)
        && tc.is_relate_to_battle(duel)
    {
        duel.send_to_hand(tc, PLAYER_NONE, REASON_EFFECT)?;
    }
    Ok(())
}
