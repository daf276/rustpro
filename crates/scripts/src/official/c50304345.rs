// Evil HERO Infernal Prodigy
use crate::prelude::*;
use crate::testing::{Check, HAND, MZONE};

const ID: u32 = 50304345;

// In the hand and on the field, with Destiny HERO - Malicious (Level 6) in the hand to Tribute Summon with it.
pub const SUPPORT: &[(u32, u16)] = &[(ID, HAND), (ID, MZONE), (9411399, HAND)];
pub const CHECKS: &[Check] = &[Check::SpecialSummoned, Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_code(duel, EFFECT_SPSUMMON_PROC);
    e1.set_property(duel, EFFECT_FLAG_SPSUM_PARAM);
    e1.set_target_range(duel, POS_FACEUP_ATTACK, 0);
    e1.set_range(duel, LOCATION_HAND);
    e1.set_condition(duel, spcon);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_CONTINUOUS);
    e2.set_code(duel, EVENT_BE_MATERIAL);
    e2.set_operation(duel, regop);
    c.register_effect(duel, e2);
}

fn spcon(duel: &Duel, a: &Args) -> bool {
    let Some(c) = a.c else { return true };
    let tp = c.get_controler(duel);
    duel.get_location_count(tp, LOCATION_MZONE) > 0 && duel.get_field_group_count(tp, LOCATION_MZONE, 0) == 0
}

fn regop(duel: &mut Duel, a: &Args) -> Script<()> {
    let c = a.e.get_handler(duel);
    let is_hero = c.get_reason_card(duel).is_some_and(|rc| rc.is_set_card(duel, &[SET_HERO]));
    if a.r == REASON_SUMMON && is_hero {
        let e1 = duel.create_effect(c);
        e1.set_description(duel, aux::stringid(ID, 0));
        e1.set_category(duel, CATEGORY_DRAW);
        e1.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_F);
        e1.set_code(duel, EVENT_PHASE + PHASE_END);
        e1.set_count_limit(duel, 1, 0);
        e1.set_range(duel, LOCATION_GRAVE);
        e1.set_target(duel, drtarget);
        e1.set_operation(duel, droperation);
        e1.set_reset(duel, RESETS_STANDARD_PHASE_END);
        c.register_effect(duel, e1);
    }
    Ok(())
}

fn drtarget(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(true);
    }
    duel.set_target_player(tp);
    duel.set_target_param(1);
    duel.set_operation_info(0, CATEGORY_DRAW, None, 0, tp, 1);
    Ok(false)
}

fn droperation(duel: &mut Duel, _: &Args) -> Script<()> {
    let p: u8 = duel.get_chain_info(0, CHAININFO_TARGET_PLAYER);
    duel.draw(p, 1, REASON_EFFECT)?;
    Ok(())
}
