// X-Saber Airbellum
use crate::prelude::*;
use crate::testing::{Check, HAND, MZONE, OPPONENT};

const ID: u32 = 90508760;

// Raigeki in hand to clear the opponent for a direct attack.
pub const SUPPORT: &[(u32, u16)] = &[(ID, MZONE), (12580477, HAND), (12580477, HAND), (83746708, HAND | OPPONENT)];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_HANDES);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_F);
    e1.set_code(duel, EVENT_BATTLE_DAMAGE);
    e1.set_condition(duel, condition);
    e1.set_target(duel, target);
    e1.set_operation(duel, operation);
    c.register_effect(duel, e1);
}

fn condition(duel: &Duel, a: &Args) -> bool {
    a.ep != a.tp && duel.get_attack_target().is_none()
}

fn target(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(true);
    }
    duel.set_operation_info(0, CATEGORY_HANDES, None, 0, 1 - a.tp, 1);
    Ok(false)
}

fn operation(duel: &mut Duel, a: &Args) -> Script<()> {
    let g = duel.get_field_group(a.ep, LOCATION_HAND, 0);
    if g.is_empty() {
        return Ok(());
    }
    let sg = g.random_select(duel, 1 - a.tp, 1);
    duel.send_to_grave(sg, REASON_DISCARD | REASON_EFFECT)?;
    Ok(())
}
