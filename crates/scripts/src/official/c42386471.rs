// Toon Gemini Elf
use super::c21296502::dircon;
use super::c53183600::{atklimit, sdescon, sdesop};
use crate::prelude::*;
use crate::testing::{Check, HAND, MZONE, OPPONENT};

const ID: u32 = 42386471;

pub const LISTED_NAMES: &[u32] = &[CARD_TOON_WORLD];

// On the field to attack the opponent's Sangans for battle damage, and Toon World in the hand to attack directly.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (ID, MZONE),
    (ID, MZONE),
    (26202165, MZONE | OPPONENT),
    (26202165, MZONE | OPPONENT),
    (CARD_TOON_WORLD, HAND),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_CONTINUOUS);
    e1.set_code(duel, EVENT_SUMMON_SUCCESS);
    e1.set_operation(duel, atklimit);
    c.register_effect(duel, e1);
    let e2 = e1.clone_effect(duel);
    e2.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    c.register_effect(duel, e2);
    let e3 = e1.clone_effect(duel);
    e3.set_code(duel, EVENT_FLIP_SUMMON_SUCCESS);
    c.register_effect(duel, e3);
    let e4 = duel.create_effect(c);
    e4.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_CONTINUOUS);
    e4.set_range(duel, LOCATION_MZONE);
    e4.set_code(duel, EVENT_LEAVE_FIELD);
    e4.set_condition(duel, sdescon);
    e4.set_operation(duel, sdesop);
    c.register_effect(duel, e4);
    let e5 = duel.create_effect(c);
    e5.set_type(duel, EFFECT_TYPE_SINGLE);
    e5.set_code(duel, EFFECT_DIRECT_ATTACK);
    e5.set_condition(duel, dircon);
    c.register_effect(duel, e5);
    let e6 = duel.create_effect(c);
    e6.set_description(duel, aux::stringid(ID, 0));
    e6.set_category(duel, CATEGORY_HANDES);
    e6.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e6.set_code(duel, EVENT_BATTLE_DAMAGE);
    e6.set_condition(duel, |_, a| a.ep != a.tp);
    e6.set_target(duel, target);
    e6.set_operation(duel, operation);
    c.register_effect(duel, e6);
}

fn target(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(duel.get_field_group_count(a.ep, LOCATION_HAND, 0) > 0);
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
    duel.send_to_grave(sg, REASON_EFFECT | REASON_DISCARD)?;
    Ok(())
}
