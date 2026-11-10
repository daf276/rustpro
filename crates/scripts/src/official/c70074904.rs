// D.D. Assailant
use crate::prelude::*;
use crate::testing::{MZONE, OPPONENT};

const ID: u32 = 70074904;

// Copies on the field to be destroyed in battle by the opponent's stronger monsters.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (ID, MZONE),
    (ID, MZONE),
    (70781052, MZONE | OPPONENT),
    (70781052, MZONE | OPPONENT),
    (13597785, MZONE | OPPONENT),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_REMOVE);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_F);
    e1.set_code(duel, EVENT_BATTLED);
    e1.set_condition(duel, condition);
    e1.set_target(duel, target);
    e1.set_operation(duel, operation);
    c.register_effect(duel, e1);
}

fn condition(duel: &Duel, a: &Args) -> bool {
    let c = a.e.get_handler(duel);
    c.is_status(duel, STATUS_BATTLE_DESTROYED) && c.is_status(duel, STATUS_OPPO_BATTLE)
}

fn target(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let c = a.e.get_handler(duel);
    let attacker = duel.get_attacker();
    let defender = duel.get_attack_target();
    if chk == 0 {
        return Ok((defender == Some(c) && attacker.is_some_and(|a| a.is_able_to_remove(duel)))
            || (attacker == Some(c) && defender.is_some_and(|t| t.is_able_to_remove(duel))));
    }
    let g: CardSet = [attacker, defender]
        .into_iter()
        .flatten()
        .filter(|c| c.is_relate_to_battle(duel))
        .collect();
    duel.set_operation_info(0, CATEGORY_REMOVE, Some(g), g.len() as u8, 0, 0);
    Ok(false)
}

fn operation(duel: &mut Duel, _: &Args) -> Script<()> {
    let g: CardSet = [duel.get_attacker(), duel.get_attack_target()]
        .into_iter()
        .flatten()
        .filter(|c| c.is_relate_to_battle(duel))
        .collect();
    duel.remove(g, POS_FACEUP, REASON_EFFECT)?;
    Ok(())
}
