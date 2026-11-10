// Rocket Warrior
use crate::prelude::*;
use crate::testing::{Check, MZONE, OPPONENT};

const ID: u32 = 30860696;

// On the field, to attack the opponent's Sangan.
pub const SUPPORT: &[(u32, u16)] = &[(ID, MZONE), (ID, MZONE), (26202165, MZONE | OPPONENT), (26202165, MZONE | OPPONENT)];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_code(duel, EFFECT_INDESTRUCTABLE_BATTLE);
    e1.set_condition(duel, ivcon);
    e1.set_value(duel, 1);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_SINGLE);
    e2.set_code(duel, EFFECT_AVOID_BATTLE_DAMAGE);
    e2.set_condition(duel, ivcon);
    e2.set_value(duel, 1);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 0));
    e3.set_category(duel, CATEGORY_ATKCHANGE);
    e3.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_F);
    e3.set_code(duel, EVENT_BATTLED);
    e3.set_condition(duel, racon);
    e3.set_operation(duel, raop);
    c.register_effect(duel, e3);
}

fn ivcon(duel: &Duel, a: &Args) -> bool {
    duel.is_turn_player(a.e.get_handler_player(duel))
}

fn racon(duel: &Duel, a: &Args) -> bool {
    Some(a.e.get_handler(duel)) == duel.get_attacker() && duel.get_attack_target().is_some()
}

fn raop(duel: &mut Duel, a: &Args) -> Script<()> {
    let Some(d) = duel.get_attack_target() else { return Ok(()) };
    if !d.is_relate_to_battle(duel) || d.is_facedown(duel) {
        return Ok(());
    }
    let e1 = duel.create_effect(a.e.get_handler(duel));
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_code(duel, EFFECT_UPDATE_ATTACK);
    e1.set_value(duel, -500);
    e1.set_reset(duel, RESETS_STANDARD_PHASE_END);
    d.register_effect(duel, e1);
    Ok(())
}
