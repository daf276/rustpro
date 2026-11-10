// Snake-Eyes Flamberge Dragon
use super::c27260347::stplace;
use crate::prelude::*;
use crate::testing::{Check, GRAVE, HAND, MZONE, OPPONENT};

const ID: u32 = 48452496;
const SNAKE_EYES_POPLAR: u32 = 90241276;
const SNAKE_EYE_ASH: u32 = 9674034;

// On the field and in the hand and Graveyard, with Level 1 FIRE Snake-Eye monsters to place, Special Summon from the
// Graveyard and Special Summon from the sent copy, and an opponent's monster to place.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (ID, HAND),
    (SNAKE_EYES_POPLAR, GRAVE),
    (SNAKE_EYES_POPLAR, GRAVE),
    (SNAKE_EYE_ASH, GRAVE),
    (SNAKE_EYE_ASH, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
    Check::ResolvesEffect(aux::stringid(ID, 2)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_count_limit(duel, 1, ID);
    e1.set_target(duel, pltg);
    e1.set_operation(duel, plop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e2.set_type(duel, EFFECT_TYPE_QUICK_O);
    e2.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e2.set_code(duel, EVENT_FREE_CHAIN);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_hint_timing(duel, 0, TIMINGS_CHECK_MONSTER_E);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_condition(duel, |duel, a| duel.is_turn_player(1 - a.tp));
    e2.set_target(duel, sptg);
    e2.set_operation(duel, spop);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 2));
    e3.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e3.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e3.set_property(duel, EFFECT_FLAG_DELAY);
    e3.set_code(duel, EVENT_TO_GRAVE);
    e3.set_count_limit(duel, 1, (ID, 2));
    e3.set_condition(duel, |duel, a| {
        a.e.get_handler(duel).is_previous_location(duel, LOCATION_ONFIELD | LOCATION_HAND)
    });
    e3.set_target(duel, gysptg);
    e3.set_operation(duel, gyspop);
    c.register_effect(duel, e3);
}

fn plfilter(c: CardId, duel: &Duel) -> bool {
    let p = c.get_owner(duel);
    c.is_faceup(duel)
        && c.is_monster(duel)
        && duel.get_location_count(p, LOCATION_SZONE) > 0
        && c.check_unique_on_field_full(duel, p, LOCATION_SZONE)
        && (c.is_location(duel, LOCATION_MZONE) || !c.is_forbidden(duel))
}

fn pltg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    let locations = LOCATION_MZONE | LOCATION_GRAVE;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, locations) && plfilter(chkc, duel));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(plfilter, tp, locations, locations, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TARGET);
    let g = duel.select_target(tp, plfilter, tp, locations, locations, 1, 1, None)?;
    if g.and_then(|g| g.get_first()).is_some_and(|tc| tc.is_location(duel, LOCATION_GRAVE)) {
        duel.set_operation_info(0, CATEGORY_LEAVE_GRAVE, g, 1, 0, 0);
    }
    Ok(false)
}

fn plop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let Some(tc) = duel.get_first_target() else { return Ok(()) };
    if !tc.is_relate_to_effect(duel, e) || tc.is_immune_to_effect(duel, e) {
        return Ok(());
    }
    if tc.is_location(duel, LOCATION_MZONE) && duel.get_location_count(tc.get_owner(duel), LOCATION_SZONE) == 0 {
        duel.send_to_grave_full(tc, REASON_RULE, PLAYER_NONE, PLAYER_NONE)?;
    } else {
        stplace(duel, tc, tp, e.get_handler(duel))?;
    }
    Ok(())
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_faceup(duel)
        && c.is_monster_card(duel)
        && c.is_continuous_spell(duel)
        && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let filter = |c, duel: &Duel| spfilter(c, duel, e, tp);
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_SZONE) && filter(chkc, duel));
    }
    if chk == 0 {
        return Ok(
            duel.get_location_count(tp, LOCATION_MZONE) > 0 && duel.is_existing_target(filter, tp, LOCATION_SZONE, LOCATION_SZONE, 1, None)
        );
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
    let g = duel.select_target(tp, filter, tp, LOCATION_SZONE, LOCATION_SZONE, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, g, 1, 0, 0);
    Ok(false)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    let Some(tc) = duel.get_first_target() else { return Ok(()) };
    if tc.is_relate_to_effect(duel, a.e) {
        duel.special_summon(tc, 0, tp, tp, false, false, POS_FACEUP)?;
    }
    Ok(())
}

fn gyspfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_level(duel, &[1]) && c.is_attribute(duel, ATTRIBUTE_FIRE) && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
}

fn gysptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if chk == 0 {
        let filter = |c, duel: &Duel| gyspfilter(c, duel, e, tp);
        return Ok(duel.is_player_affected_by_effect(tp, CARD_BLUEEYES_SPIRIT).is_none()
            && duel.get_location_count(tp, LOCATION_MZONE) > 1
            && duel.is_existing_matching_card(filter, tp, LOCATION_GRAVE, 0, 2, None));
    }
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 2, tp, LOCATION_GRAVE as i32);
    Ok(false)
}

fn gyspop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    if duel.is_player_affected_by_effect(tp, CARD_BLUEEYES_SPIRIT).is_some() || duel.get_location_count(tp, LOCATION_MZONE) <= 1 {
        return Ok(());
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
    let filter = |c, duel: &Duel| gyspfilter(c, duel, e, tp);
    let g = duel.select_matching_card(tp, filter, tp, LOCATION_GRAVE, 0, 2, 2, None)?;
    if g.len() > 1 {
        duel.special_summon(g, 0, tp, tp, false, false, POS_FACEUP)?;
    }
    Ok(())
}
