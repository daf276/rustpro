// Exploder Dragon
use crate::prelude::*;
use crate::testing::{MZONE, OPPONENT};

const ID: u32 = 20586572;

// Two on the field with Elfnote Lucina of the opponent's to attack them.
pub const SUPPORT: &[(u32, u16)] = &[(ID, MZONE), (ID, MZONE), (13597785, MZONE | OPPONENT), (13597785, MZONE | OPPONENT)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_DESTROY);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_F);
    e1.set_code(duel, EVENT_BATTLE_DESTROYED);
    e1.set_condition(duel, condition);
    e1.set_target(duel, target);
    e1.set_operation(duel, operation);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_SINGLE);
    e2.set_code(duel, EFFECT_NO_BATTLE_DAMAGE);
    e2.set_condition(duel, damcon);
    e2.set_value(duel, 1);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_type(duel, EFFECT_TYPE_SINGLE);
    e3.set_code(duel, EFFECT_AVOID_BATTLE_DAMAGE);
    e3.set_condition(duel, damcon);
    e3.set_value(duel, 1);
    c.register_effect(duel, e3);
}

fn condition(duel: &Duel, a: &Args) -> bool {
    let c = a.e.get_handler(duel);
    c.is_location(duel, LOCATION_GRAVE)
        && c.is_reason(duel, REASON_BATTLE)
        && c.get_reason_card(duel).is_some_and(|rc| rc.is_relate_to_battle(duel))
}

fn target(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(true);
    }
    if let Some(rc) = a.e.get_handler(duel).get_reason_card(duel) {
        rc.create_effect_relation(duel, a.e);
        duel.set_operation_info(0, CATEGORY_DESTROY, Some(rc.into()), 1, 0, 0);
    }
    Ok(false)
}

fn operation(duel: &mut Duel, a: &Args) -> Script<()> {
    if let Some(rc) = a.e.get_handler(duel).get_reason_card(duel)
        && rc.is_relate_to_effect(duel, a.e)
    {
        duel.destroy(rc, REASON_EFFECT)?;
    }
    Ok(())
}

fn damcon(duel: &Duel, a: &Args) -> bool {
    duel.get_attacker() == Some(a.e.get_handler(duel))
}
