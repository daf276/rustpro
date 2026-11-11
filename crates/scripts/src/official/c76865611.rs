// Morphtronic Lantron
use crate::prelude::*;
use crate::testing::{Check, HAND, MZONE, OPPONENT};

const ID: u32 = 76865611;

// Copies on the field to be destroyed in Defense Position by the opponent's Buster Bladers and Lucina, or to reflect the opponent's Meteors of Destruction.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (ID, MZONE),
    (ID, MZONE),
    (78193831, MZONE | OPPONENT),
    (78193831, MZONE | OPPONENT),
    (13597785, MZONE | OPPONENT),
    (33767325, HAND | OPPONENT),
    (33767325, HAND | OPPONENT),
    (33767325, HAND | OPPONENT),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_code(duel, EFFECT_REFLECT_DAMAGE);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_property(duel, EFFECT_FLAG_PLAYER_TARGET);
    e1.set_condition(duel, |duel, a| a.e.get_handler(duel).is_attack_pos(duel));
    e1.set_target_range(duel, 1, 0);
    e1.set_value(duel, Value::ReflectDamage(refval));
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_CONTINUOUS);
    e2.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE);
    e2.set_code(duel, EVENT_LEAVE_FIELD_P);
    e2.set_operation(duel, check);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 0));
    e3.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_F);
    e3.set_code(duel, EVENT_BATTLE_DESTROYED);
    e3.set_condition(duel, cond);
    e3.set_operation(duel, opd);
    e3.set_label_object(duel, Some(e2));
    c.register_effect(duel, e3);
}

fn refval(duel: &Duel, e: EffectId, _: Option<EffectId>, _: u32, r: u32, rp: u8, _: Option<CardId>) -> bool {
    r & REASON_EFFECT != 0 && rp == 1 - e.get_handler_player(duel)
}

fn check(duel: &mut Duel, a: &Args) -> Script<()> {
    let c = a.e.get_handler(duel);
    let label = i64::from(!c.is_disabled(duel) && c.is_defense_pos(duel));
    a.e.set_label(duel, label);
    Ok(())
}

fn cond(duel: &Duel, a: &Args) -> bool {
    let c = a.e.get_handler(duel);
    a.e.get_label_object(duel).is_some_and(|e| e.get_label(duel) == 1)
        && c.is_location(duel, LOCATION_GRAVE)
        && c.is_reason(duel, REASON_BATTLE)
}

fn opd(duel: &mut Duel, a: &Args) -> Script<()> {
    let e1 = duel.create_effect(a.e.get_handler(duel));
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_code(duel, EFFECT_AVOID_BATTLE_DAMAGE);
    e1.set_property(duel, EFFECT_FLAG_PLAYER_TARGET);
    e1.set_target_range(duel, 1, 0);
    e1.set_value(duel, 1);
    e1.set_reset(duel, RESET_PHASE | PHASE_END);
    duel.register_effect(e1, a.tp);
    Ok(())
}
