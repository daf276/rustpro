// D.D. Warrior Lady
use crate::prelude::*;
use crate::testing::{MZONE, OPPONENT};

const ID: u32 = 7572887;

// On the field against the opponent's Normal Monsters, to attack and be attacked by.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (50930991, MZONE | OPPONENT),
    (50930991, MZONE | OPPONENT),
    (50930991, MZONE | OPPONENT),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_REMOVE);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e1.set_code(duel, EVENT_BATTLED);
    e1.set_target(duel, target);
    e1.set_operation(duel, operation);
    c.register_effect(duel, e1);
}

fn target(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let c = a.e.get_handler(duel);
    let (attacker, target) = (duel.get_attacker(), duel.get_attack_target());
    if chk == 0 {
        return Ok(
            (target == Some(c) && attacker.is_some_and(|attacker| attacker.is_able_to_remove(duel)))
                || (attacker == Some(c) && target.is_some_and(|target| target.is_able_to_remove(duel))),
        );
    }
    let g: CardSet = [attacker, target]
        .into_iter()
        .flatten()
        .filter(|bc| bc.is_relate_to_battle(duel))
        .collect();
    duel.set_operation_info(0, CATEGORY_REMOVE, Some(g), g.len() as u8, 0, 0);
    Ok(false)
}

fn operation(duel: &mut Duel, _: &Args) -> Script<()> {
    let g: CardSet = [duel.get_attacker(), duel.get_attack_target()].into_iter().flatten().collect();
    let rg = g.filter(duel, CardId::is_relate_to_battle, None);
    duel.remove(rg, POS_FACEUP, REASON_EFFECT)?;
    Ok(())
}
