// Blackwing - Jin the Rain Shadow
use crate::prelude::*;
use crate::testing::{Check, MZONE, OPPONENT};

const ID: u32 = 38562933;

// Copies on the field, with Level 3 Normal Monsters of the opponent's (0 DEF) to battle.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (ID, MZONE),
    (22916281, MZONE | OPPONENT),
    (22916281, MZONE | OPPONENT),
    (22916281, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_DESTROY);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_F);
    e1.set_code(duel, EVENT_BATTLE_START);
    e1.set_condition_mut(duel, descon);
    e1.set_target(duel, destg);
    e1.set_operation(duel, desop);
    c.register_effect(duel, e1);
}

fn descon(duel: &mut Duel, a: &Args) -> bool {
    let c = a.e.get_handler(duel);
    let mut d = duel.get_attack_target();
    if d == Some(c) {
        d = duel.get_attacker();
    }
    a.e.set_label_object_group(duel, d.map(CardSet::from));
    d.is_some_and(|d| d.is_faceup(duel) && d.is_defense_below(duel, c.get_attack(duel)))
}

fn destg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(true);
    }
    let g = a.e.get_label_object_group(duel);
    duel.set_operation_info(0, CATEGORY_DESTROY, g, 1, 0, 0);
    Ok(false)
}

fn desop(duel: &mut Duel, a: &Args) -> Script<()> {
    let c = a.e.get_handler(duel);
    if let Some(d) = a.e.get_label_object_group(duel).and_then(|g| g.get_first())
        && c.is_faceup(duel)
        && c.is_relate_to_effect(duel, a.e)
        && d.is_relate_to_battle(duel)
        && d.is_defense_below(duel, c.get_attack(duel))
    {
        duel.destroy(d, REASON_EFFECT)?;
    }
    Ok(())
}
