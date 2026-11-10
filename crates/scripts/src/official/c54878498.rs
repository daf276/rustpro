// Kelbek
use crate::prelude::*;
use crate::testing::{MZONE, OPPONENT};

const ID: u32 = 54878498;

// Copies on the field to be attacked by the opponent's Sangan and Elfnote Lucina.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (ID, MZONE),
    (ID, MZONE),
    (26202165, MZONE | OPPONENT),
    (13597785, MZONE | OPPONENT),
    (13597785, MZONE | OPPONENT),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_TOHAND);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_F);
    e1.set_code(duel, EVENT_BATTLED);
    e1.set_condition(duel, condition);
    e1.set_target(duel, target);
    e1.set_operation(duel, operation);
    c.register_effect(duel, e1);
}

fn condition(duel: &Duel, a: &Args) -> bool {
    let attacker = duel.get_attacker();
    unported!(duel, attacker.is_none());
    duel.get_attack_target() == Some(a.e.get_handler(duel)) && attacker.is_some_and(|at| !at.is_status(duel, STATUS_BATTLE_DESTROYED))
}

fn target(duel: &mut Duel, _: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(true);
    }
    duel.set_operation_info(0, CATEGORY_TOHAND, duel.get_attacker().map(CardSet::from), 1, 0, 0);
    Ok(false)
}

fn operation(duel: &mut Duel, _: &Args) -> Script<()> {
    let attacker = duel.get_attacker();
    unported!(duel, attacker.is_none());
    if let Some(a) = attacker
        && a.is_relate_to_battle(duel)
    {
        duel.send_to_hand(a, PLAYER_NONE, REASON_EFFECT)?;
    }
    Ok(())
}
