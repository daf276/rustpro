// Robbin' Goblin
use crate::prelude::*;
use crate::testing::{Check, MZONE, SZONE};

const ID: u32 = 88279736;

// Set with Goblin Attack Force, 2300 ATK, on the field to attack with.
pub const SUPPORT: &[(u32, u16)] = &[(ID, SZONE), (ID, SZONE), (78658564, MZONE), (78658564, MZONE)];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 0));
    e2.set_category(duel, CATEGORY_HANDES);
    e2.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_F);
    e2.set_range(duel, LOCATION_SZONE);
    e2.set_code(duel, EVENT_BATTLE_DAMAGE);
    e2.set_condition(duel, condition);
    e2.set_target(duel, target);
    e2.set_operation(duel, operation);
    c.register_effect(duel, e2);
}

fn condition(duel: &Duel, a: &Args) -> bool {
    a.ep != a.tp && a.eg.and_then(|eg| eg.get_first()).is_some_and(|c| c.is_controler(duel, a.tp))
}

fn target(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(true);
    }
    duel.set_operation_info(0, CATEGORY_HANDES, None, 0, 1 - a.tp, 1);
    Ok(false)
}

fn operation(duel: &mut Duel, a: &Args) -> Script<()> {
    if !a.e.get_handler(duel).is_relate_to_effect(duel, a.e) {
        return Ok(());
    }
    let g = duel.get_field_group(a.ep, LOCATION_HAND, 0);
    let sg = g.random_select(duel, a.ep, 1);
    duel.send_to_grave(sg, REASON_DISCARD | REASON_EFFECT)?;
    Ok(())
}
