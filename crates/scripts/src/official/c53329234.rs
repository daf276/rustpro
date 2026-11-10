// Sinful Spoils of Slumber - Morrian
use crate::prelude::*;
use crate::testing::{Check, GRAVE, HAND, MZONE, OPPONENT, SZONE};

const ID: u32 = 53329234;

// Set, in the hand and in the Graveyard, with Pesanta the Titan of Troubadours (a Level 8 Illusion) on the field and Special
// Summoned monsters of both players (Jurrac Megalo) to turn face-down.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, SZONE),
    (ID, GRAVE),
    (66429798, MZONE),
    (66429798, MZONE),
    (93170499, HAND),
    (93170499, MZONE),
    (93170499, HAND | OPPONENT),
    (93170499, MZONE | OPPONENT),
    (93170499, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_POSITION + CATEGORY_SET);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_count_limit(duel, 1, ID);
    e1.set_hint_timing(duel, TIMING_BATTLE_PHASE, TIMINGS_CHECK_MONSTER_E | TIMING_BATTLE_PHASE);
    e1.set_target(duel, postg);
    e1.set_operation(duel, posop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_SET);
    e2.set_type(duel, EFFECT_TYPE_QUICK_O);
    e2.set_code(duel, EVENT_FREE_CHAIN);
    e2.set_range(duel, LOCATION_GRAVE);
    e2.set_count_limit(duel, 1, ID);
    e2.set_hint_timing(duel, TIMING_END_PHASE, TIMING_BATTLE_END | TIMINGS_CHECK_MONSTER_E);
    e2.set_condition(duel, setcon);
    e2.set_target(duel, settg);
    e2.set_operation(duel, setop);
    c.register_effect(duel, e2);
}

fn posfilter(c: CardId, duel: &Duel) -> bool {
    c.is_faceup(duel) && c.is_special_summoned(duel) && c.is_can_turn_set(duel)
}

fn postg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_MZONE) && posfilter(chkc, duel));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(posfilter, tp, LOCATION_MZONE, LOCATION_MZONE, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_FACEUP);
    let g = duel.select_target(tp, posfilter, tp, LOCATION_MZONE, LOCATION_MZONE, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_POSITION, g, 1, tp, POS_FACEDOWN_DEFENSE as i32);
    Ok(false)
}

fn posop(duel: &mut Duel, a: &Args) -> Script<()> {
    if let Some(tc) = duel.get_first_target()
        && tc.is_relate_to_effect(duel, a.e)
        && tc.is_faceup(duel)
    {
        duel.change_position(tc, POS_FACEDOWN_DEFENSE)?;
    }
    Ok(())
}

fn setconfilter(c: CardId, duel: &Duel) -> bool {
    c.is_faceup(duel) && c.is_race(duel, RACE_ILLUSION) && c.is_level_above(duel, 5)
}

fn setcon(duel: &Duel, a: &Args) -> bool {
    duel.is_existing_matching_card(setconfilter, a.tp, LOCATION_MZONE, 0, 1, None)
}

fn settg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let c = a.e.get_handler(duel);
    if chk == 0 {
        return Ok(c.is_ssetable(duel));
    }
    duel.set_operation_info(0, CATEGORY_LEAVE_GRAVE, Some(c.into()), 1, 0, 0);
    Ok(false)
}

fn setop(duel: &mut Duel, a: &Args) -> Script<()> {
    let c = a.e.get_handler(duel);
    if c.is_relate_to_effect(duel, a.e) && c.is_ssetable(duel) && duel.sset(a.tp, c)? > 0 {
        let e1 = duel.create_effect(c);
        e1.set_description(duel, 3300);
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE + EFFECT_FLAG_CLIENT_HINT);
        e1.set_code(duel, EFFECT_LEAVE_FIELD_REDIRECT);
        e1.set_value(duel, LOCATION_REMOVED as i32);
        e1.set_reset(duel, RESET_EVENT | RESETS_REDIRECT);
        c.register_effect(duel, e1);
    }
    Ok(())
}
