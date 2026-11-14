// Mimighoul Armor
use crate::prelude::*;
use crate::testing::{Check, DECK, HAND, MZONE, OPPONENT};

const ID: u32 = 11677278;

pub const LISTED_SERIES: &[u16] = &[SET_MIMIGHOUL];
// Copies face-down on the field to Flip with the opponent's Elfnote Lucina to attack them, and in the hand to Special Summon.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (ID, MZONE),
    (ID, MZONE),
    (ID, MZONE),
    (ID, HAND),
    (13597785, MZONE | OPPONENT),
    (53129443, DECK),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_CONTROL);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_FLIP + EFFECT_TYPE_TRIGGER_F);
    e1.set_count_limit(duel, 1, ID);
    e1.set_condition(duel, |duel, _| duel.is_main_phase());
    e1.set_target(duel, efftg);
    e1.set_operation(duel, effop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_SPECIAL_SUMMON + CATEGORY_SET);
    e2.set_type(duel, EFFECT_TYPE_IGNITION);
    e2.set_range(duel, LOCATION_HAND);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_target(duel, selfsptg);
    e2.set_operation(duel, selfspop);
    c.register_effect(duel, e2);
}

fn efftg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(true);
    }
    let c = a.e.get_handler(duel);
    if c.is_able_to_change_controler(duel) {
        duel.set_operation_info(0, CATEGORY_CONTROL, Some(c.into()), 1, a.tp, 0);
    }
    Ok(false)
}

fn effop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_code(duel, EFFECT_INDESTRUCTABLE_BATTLE);
    e1.set_target_range(duel, LOCATION_MZONE, LOCATION_MZONE);
    e1.set_target_card(duel, |duel, _, c| c.is_set_card(duel, &[SET_MIMIGHOUL]));
    e1.set_value(duel, 1);
    e1.set_reset(duel, RESET_PHASE | PHASE_END);
    duel.register_effect(e1, tp);
    aux::register_client_hint(duel, c, 0, tp, 1, 1, Some(aux::stringid(ID, 2)));
    if c.is_relate_to_effect(duel, e) {
        duel.break_effect()?;
        duel.get_control(c, 1 - tp)?;
    }
    Ok(())
}

pub fn can_set_to_opponent(duel: &Duel, e: EffectId, tp: u8) -> bool {
    duel.get_location_count_full(1 - tp, LOCATION_MZONE, tp, LOCATION_REASON_TOFIELD, 0xff) > 0
        && e.get_handler(duel)
            .is_can_be_special_summoned_full(duel, e, 0, tp, false, false, POS_FACEDOWN_DEFENSE, 1 - tp, 0xff)
}

fn can_summon_to_self(duel: &Duel, e: EffectId, tp: u8, condition: fn(&Duel, u8) -> bool) -> bool {
    condition(duel, tp)
        && duel.get_location_count(tp, LOCATION_MZONE) > 0
        && e.get_handler(duel).is_can_be_special_summoned(duel, e, 0, tp, false, false)
}

fn opponent_controls_monster(duel: &Duel, tp: u8) -> bool {
    duel.get_field_group_count(tp, 0, LOCATION_MZONE) > 0
}

fn selfsptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    selfsptg_with(duel, a, chk, opponent_controls_monster)
}

fn selfspop(duel: &mut Duel, a: &Args) -> Script<()> {
    selfspop_with(duel, a, ID, opponent_controls_monster)
}

// The hand effect of the Mimighoul monsters that can also Special Summon themselves to their controller's field on a condition.
pub fn selfsptg_with(duel: &mut Duel, a: &Args, chk: i32, condition: fn(&Duel, u8) -> bool) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if chk == 0 {
        return Ok(can_set_to_opponent(duel, e, tp) || can_summon_to_self(duel, e, tp, condition));
    }
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, Some(e.get_handler(duel).into()), 1, tp, 0);
    Ok(false)
}

pub fn selfspop_with(duel: &mut Duel, a: &Args, id: u32, condition: fn(&Duel, u8) -> bool) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if !c.is_relate_to_effect(duel, e) {
        return Ok(());
    }
    let (b1, b2) = (can_set_to_opponent(duel, e, tp), can_summon_to_self(duel, e, tp, condition));
    let op = if b1 && b2 {
        duel.select_effect(tp, &[(b1, aux::stringid(id, 3)), (b2, aux::stringid(id, 4))])?
    } else {
        Some(if b1 || !b2 { 1 } else { 2 })
    };
    if op == Some(1) {
        if duel.special_summon(c, 0, tp, 1 - tp, false, false, POS_FACEDOWN_DEFENSE)? == 0 {
            return Ok(());
        }
        duel.confirm_cards(tp, c)?;
    } else if op == Some(2) {
        duel.special_summon(c, 0, tp, tp, false, false, POS_FACEUP)?;
    }
    Ok(())
}
