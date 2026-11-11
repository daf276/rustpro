// Karakuri Barrel mdl 96 "Shinkuro"
use crate::prelude::*;
use crate::testing::{Check, HAND, MZONE, OPPONENT};

const ID: u32 = 92300891;

// Copies on the field and in the hand, with Mystical Elf on the opponent's side to attack them.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (ID, MZONE),
    (ID, MZONE),
    (ID, HAND),
    (15025844, MZONE | OPPONENT),
    (15025844, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0))];

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
    e3.set_condition(duel, poscon);
    e3.set_operation(duel, posop);
    c.register_effect(duel, e3);
    let e4 = duel.create_effect(c);
    e4.set_type(duel, EFFECT_TYPE_SINGLE);
    e4.set_property(duel, EFFECT_FLAG_SINGLE_RANGE);
    e4.set_range(duel, LOCATION_MZONE);
    e4.set_code(duel, EFFECT_INDESTRUCTABLE_COUNT);
    e4.set_count_limit(duel, 1, 0);
    e4.set_value(duel, Value::Reason(|_, _, _, r, _| r & REASON_BATTLE != 0));
    c.register_effect(duel, e4);
}

fn poscon(duel: &Duel, a: &Args) -> bool {
    a.e.get_handler(duel).is_attack_pos(duel)
}

fn posop(duel: &mut Duel, a: &Args) -> Script<()> {
    let c = a.e.get_handler(duel);
    if c.is_faceup(duel) && c.is_relate_to_effect(duel, a.e) {
        duel.change_position(c, POS_FACEUP_DEFENSE)?;
    }
    Ok(())
}
