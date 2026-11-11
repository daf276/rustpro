// Witchcrafter Unveiling
use crate::prelude::*;
use crate::testing::{Check, GRAVE, HAND, MZONE};

const ID: u32 = 70226289;

// In the hand and the Graveyard, with Witchcrafter Genni and Schmietta in the hand to summon and on the field to recycle it.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, GRAVE),
    (64756282, HAND),
    (21744288, HAND),
    (84523092, HAND),
    (21744288, MZONE),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(0)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_hint_timing(duel, 0, TIMING_END_PHASE);
    e1.set_count_limit(duel, 1, ID);
    e1.set_target(duel, target);
    e1.set_operation(duel, activate);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_category(duel, CATEGORY_TOHAND + CATEGORY_LEAVE_GRAVE);
    e2.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e2.set_code(duel, EVENT_PHASE + PHASE_END);
    e2.set_range(duel, LOCATION_GRAVE);
    e2.set_count_limit(duel, 1, ID);
    e2.set_condition(duel, thcon);
    e2.set_target(duel, thtg);
    e2.set_operation(duel, thop);
    c.register_effect(duel, e2);
}

fn filter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_set_card(duel, &[SET_WITCHCRAFTER]) && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
}

fn target(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    stock::special_summon_target(duel, a, chk, |c, duel| filter(c, duel, a.e, a.tp), LOCATION_HAND)
}

fn activate(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    if duel.get_location_count(tp, LOCATION_MZONE) <= 0 {
        return Ok(());
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
    let g = duel.select_matching_card(tp, |c, duel| filter(c, duel, e, tp), tp, LOCATION_HAND, 0, 1, 1, None)?;
    if duel.special_summon(g, 0, tp, tp, false, false, POS_FACEUP)? != 0 {
        let c = e.get_handler(duel);
        let e1 = duel.create_effect(c);
        e1.set_description(duel, aux::stringid(ID, 1));
        e1.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_CONTINUOUS);
        e1.set_code(duel, EVENT_CHAINING);
        e1.set_operation(duel, actop);
        e1.set_reset(duel, RESET_PHASE | PHASE_END);
        duel.register_effect(e1, tp);
        let e2 = duel.create_effect(c);
        e2.set_property(duel, EFFECT_FLAG_PLAYER_TARGET + EFFECT_FLAG_CLIENT_HINT);
        e2.set_description(duel, aux::stringid(ID, 1));
        e2.set_reset(duel, RESET_PHASE | PHASE_END);
        e2.set_target_range(duel, 1, 0);
        duel.register_effect(e2, tp);
    }
    Ok(())
}

fn actop(duel: &mut Duel, a: &Args) -> Script<()> {
    if let Some(re) = a.re
        && re.get_handler(duel).is_race(duel, RACE_SPELLCASTER)
        && re.is_monster_effect(duel)
        && a.ep == a.tp
    {
        duel.set_chain_limit(|_, _, rp, tp, _| tp == rp, 0);
    }
    Ok(())
}

fn thcon(duel: &Duel, a: &Args) -> bool {
    let tp = a.tp;
    duel.is_existing_matching_card(
        aux::faceup_filter(|c, duel| c.is_set_card(duel, &[SET_WITCHCRAFTER])),
        tp,
        LOCATION_MZONE,
        0,
        1,
        None,
    ) && duel.is_turn_player(tp)
}

fn thtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let c = a.e.get_handler(duel);
    if chk == 0 {
        return Ok(c.is_able_to_hand(duel));
    }
    duel.set_operation_info(0, CATEGORY_TOHAND, Some(c.into()), 1, a.tp, LOCATION_GRAVE as i32);
    Ok(false)
}

fn thop(duel: &mut Duel, a: &Args) -> Script<()> {
    let c = a.e.get_handler(duel);
    if c.is_relate_to_effect(duel, a.e) {
        duel.send_to_hand(c, a.tp, REASON_EFFECT)?;
    }
    Ok(())
}
