// Blue-Eyes Toon Dragon
use engine::constants::MSG_ATTACK;

use crate::prelude::*;
use crate::testing::{Check, HAND, MZONE, OPPONENT, SZONE};

const ID: u32 = 53183600;

pub const LISTED_NAMES: &[u32] = &[CARD_TOON_WORLD];

// In the hand, Special Summoned by Tributing two Normal Monsters with Toon World on the field, to attack the opponent's
// monsters.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (CARD_TOON_WORLD, SZONE),
    (89631139, MZONE),
    (89631139, MZONE),
    (26202165, MZONE | OPPONENT),
];
// It has no effect to resolve.
pub const CHECKS: &[Check] = &[Check::SpecialSummoned, Check::Message(MSG_ATTACK)];

fn release_options(c: CardId, tp: u8) -> ReleaseOptions {
    ReleaseOptions {
        check_field: true,
        to_check: Some(c),
        ..ReleaseOptions::new(tp)
    }
}

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    initial_effect_releasing::<2>(duel, c);
}

pub fn initial_effect_releasing<const COUNT: u8>(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_code(duel, EFFECT_SPSUMMON_PROC);
    e1.set_property(duel, EFFECT_FLAG_UNCOPYABLE);
    e1.set_range(duel, LOCATION_HAND);
    e1.set_condition(duel, spcon::<COUNT>);
    e1.set_target(duel, sptg::<COUNT>);
    e1.set_operation(duel, spop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_CONTINUOUS);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_code(duel, EVENT_LEAVE_FIELD);
    e2.set_condition(duel, sdescon);
    e2.set_operation(duel, sdesop);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_type(duel, EFFECT_TYPE_SINGLE);
    e3.set_code(duel, EFFECT_DIRECT_ATTACK);
    e3.set_condition(duel, dircon);
    c.register_effect(duel, e3);
    let e4 = duel.create_effect(c);
    e4.set_type(duel, EFFECT_TYPE_SINGLE);
    e4.set_code(duel, EFFECT_CANNOT_SELECT_BATTLE_TARGET);
    e4.set_condition(duel, atcon);
    e4.set_value(duel, Value::Card(|duel, _, c| i32::from(atlimit(duel, c))));
    c.register_effect(duel, e4);
    let e5 = duel.create_effect(c);
    e5.set_type(duel, EFFECT_TYPE_SINGLE);
    e5.set_code(duel, EFFECT_CANNOT_DIRECT_ATTACK);
    e5.set_condition(duel, atcon);
    c.register_effect(duel, e5);
    let e6 = duel.create_effect(c);
    e6.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_CONTINUOUS);
    e6.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e6.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE);
    e6.set_operation(duel, atklimit);
    c.register_effect(duel, e6);
    let e7 = duel.create_effect(c);
    e7.set_type(duel, EFFECT_TYPE_SINGLE);
    e7.set_code(duel, EFFECT_ATTACK_COST);
    e7.set_cost_condition(duel, |duel, _, _, tp, _| duel.check_lp_cost(tp, 500));
    e7.set_operation(duel, atop);
    c.register_effect(duel, e7);
}

pub fn toon_world_on_field(duel: &Duel, tp: u8) -> bool {
    duel.is_existing_matching_card(
        aux::faceup_filter(|c, duel| c.is_code(duel, CARD_TOON_WORLD)),
        tp,
        LOCATION_ONFIELD,
        0,
        1,
        None,
    )
}

pub fn spcon<const COUNT: u8>(duel: &Duel, a: &Args) -> bool {
    let Some(c) = a.c else { return true };
    let tp = c.get_controler(duel);
    duel.check_release_group_full(tp, aux::TRUE, COUNT as usize, release_options(c, tp)) && toon_world_on_field(duel, tp)
}

pub fn sptg<const COUNT: u8>(duel: &mut Duel, a: &Args, _: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    let Some(c) = a.c else { return Ok(false) };
    let g = duel.select_release_group_full(tp, aux::TRUE, (COUNT, COUNT), true, release_options(c, tp))?;
    if g.is_empty() {
        return Ok(false);
    }
    a.e.set_label_object_group(duel, Some(g));
    Ok(true)
}

pub fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    if let Some(g) = a.e.get_label_object_group(duel) {
        duel.release(g, REASON_COST)?;
    }
    Ok(())
}

fn sfilter(c: CardId, duel: &Duel) -> bool {
    c.is_reason(duel, REASON_DESTROY)
        && c.is_previous_position(duel, POS_FACEUP)
        && c.get_previous_code_on_field(duel) == CARD_TOON_WORLD
        && c.is_previous_location(duel, LOCATION_ONFIELD)
}

pub fn sdescon(duel: &Duel, a: &Args) -> bool {
    a.eg.is_some_and(|eg| eg.is_exists(duel, sfilter, 1, None))
}

pub fn sdesop(duel: &mut Duel, a: &Args) -> Script<()> {
    let c = a.e.get_handler(duel);
    duel.destroy(c, REASON_EFFECT)?;
    Ok(())
}

fn atkfilter(c: CardId, duel: &Duel) -> bool {
    c.is_faceup(duel) && c.is_type(duel, TYPE_TOON)
}

fn opponent_has_toon(duel: &Duel, a: &Args) -> bool {
    duel.is_existing_matching_card(atkfilter, a.e.get_handler_player(duel), 0, LOCATION_MZONE, 1, None)
}

pub fn dircon(duel: &Duel, a: &Args) -> bool {
    !opponent_has_toon(duel, a)
}

pub fn atcon(duel: &Duel, a: &Args) -> bool {
    opponent_has_toon(duel, a)
}

pub fn atlimit(duel: &Duel, c: CardId) -> bool {
    !c.is_type(duel, TYPE_TOON) || c.is_facedown(duel)
}

pub fn atklimit(duel: &mut Duel, a: &Args) -> Script<()> {
    let c = a.e.get_handler(duel);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_code(duel, EFFECT_CANNOT_ATTACK);
    e1.set_reset(duel, RESETS_STANDARD_PHASE_END);
    c.register_effect(duel, e1);
    Ok(())
}

pub fn atop(duel: &mut Duel, a: &Args) -> Script<()> {
    if duel.is_attack_cost_paid() != 2 && a.e.get_handler(duel).is_location(duel, LOCATION_MZONE) {
        duel.pay_lp_cost(a.tp, 500)?;
        duel.attack_cost_paid();
    }
    Ok(())
}
