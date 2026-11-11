// Karakuri Watchdog mdl 313 "Saizan"
use crate::prelude::*;
use crate::testing::{Check, HAND, MZONE, OPPONENT};

const ID: u32 = 70271583;

// Copies on the field and in the hand, with strong monsters of the opponent's to fight and take battle damage from.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (ID, MZONE),
    (ID, HAND),
    (ID, HAND),
    (27125110, MZONE | OPPONENT),
    (27125110, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_code(duel, EFFECT_MUST_ATTACK);
    c.register_effect(duel, e1);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 0));
    e3.set_category(duel, CATEGORY_POSITION);
    e3.set_type(duel, EFFECT_TYPE_TRIGGER_F + EFFECT_TYPE_SINGLE);
    e3.set_code(duel, EVENT_BE_BATTLE_TARGET);
    e3.set_operation(duel, posop);
    c.register_effect(duel, e3);
    let e4 = duel.create_effect(c);
    e4.set_description(duel, aux::stringid(ID, 1));
    e4.set_type(duel, EFFECT_TYPE_TRIGGER_F + EFFECT_TYPE_FIELD);
    e4.set_range(duel, LOCATION_MZONE);
    e4.set_category(duel, CATEGORY_ATKCHANGE + CATEGORY_DEFCHANGE);
    e4.set_code(duel, EVENT_BATTLE_DAMAGE);
    e4.set_condition(duel, atkcon);
    e4.set_operation(duel, atkop);
    c.register_effect(duel, e4);
    let e5 = duel.create_effect(c);
    e5.set_type(duel, EFFECT_TYPE_SINGLE);
    e5.set_code(duel, EFFECT_INDESTRUCTABLE_BATTLE);
    e5.set_condition(duel, indcon);
    e5.set_value(duel, 1);
    c.register_effect(duel, e5);
}

fn posop(duel: &mut Duel, a: &Args) -> Script<()> {
    let c = a.e.get_handler(duel);
    if c.is_faceup(duel) && c.is_relate_to_effect(duel, a.e) {
        duel.change_position_full(c, [POS_FACEUP_DEFENSE, 0, POS_FACEUP_ATTACK, 0], false)?;
    }
    Ok(())
}

fn atkcon(duel: &Duel, a: &Args) -> bool {
    let c = a.e.get_handler(duel);
    a.ep == a.tp && (Some(c) == duel.get_attacker() || Some(c) == duel.get_attack_target())
}

fn filter(c: CardId, duel: &Duel) -> bool {
    c.is_faceup(duel) && c.is_set_card(duel, &[SET_KARAKURI])
}

fn atkop(duel: &mut Duel, a: &Args) -> Script<()> {
    let c = a.e.get_handler(duel);
    let g = duel.get_matching_group(filter, a.tp, LOCATION_MZONE, 0, None);
    for tc in g {
        let e1 = duel.create_effect(c);
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE);
        e1.set_code(duel, EFFECT_UPDATE_ATTACK);
        e1.set_reset(duel, RESETS_STANDARD_PHASE_END);
        e1.set_value(duel, 800);
        tc.register_effect(duel, e1);
        let e2 = e1.clone_effect(duel);
        e2.set_code(duel, EFFECT_UPDATE_DEFENSE);
        tc.register_effect(duel, e2);
    }
    Ok(())
}

fn indcon(duel: &Duel, a: &Args) -> bool {
    a.e.get_handler(duel).is_position(duel, POS_FACEUP_ATTACK)
}
