// Swordsoul Strategist Longyuan
use crate::prelude::*;
use crate::testing::{Check, DECK, EXTRA, HAND, MZONE};

const ID: u32 = 93490856;

pub const LISTED_NAMES: &[u32] = &[TOKEN_SWORDSOUL];

// A second copy and Mystery Shell Dragon (a Wyrm) in the hand for the discard, and Chengying in the Extra Deck to Synchro Summon with it.
pub const SUPPORT: &[(u32, u16)] = &[(ID, HAND), (ID, DECK), (18108166, HAND), (18108166, MZONE), (96633955, EXTRA)];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
    Check::SpecialSummonedCode(TOKEN_SWORDSOUL),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON + CATEGORY_TOKEN);
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_range(duel, LOCATION_HAND);
    e1.set_count_limit(duel, 1, ID);
    e1.set_cost(duel, spcost);
    e1.set_target(duel, sptg);
    e1.set_operation(duel, spop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_DAMAGE);
    e2.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e2.set_code(duel, EVENT_BE_MATERIAL);
    e2.set_property(duel, EFFECT_FLAG_DELAY + EFFECT_FLAG_PLAYER_TARGET);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_condition(duel, |duel, a| {
        a.e.get_handler(duel).is_location(duel, LOCATION_GRAVE) && a.r == REASON_SYNCHRO
    });
    e2.set_target(duel, damtg);
    e2.set_operation(duel, damop);
    c.register_effect(duel, e2);
}

pub fn token_stats() -> MonsterStats {
    MonsterStats::new(SET_SWORDSOUL, TYPES_TOKEN + TYPE_TUNER, 0, 0, 4, RACE_WYRM, ATTRIBUTE_WATER)
}

// The Swordsoul Token with its Special Summon restriction, summoned by the step; the caller completes the summon.
pub fn summon_token(duel: &mut Duel, c: CardId, tp: u8) -> Script<()> {
    if let Some(token) = duel.create_token(tp, TOKEN_SWORDSOUL) {
        duel.special_summon_step(token, 0, tp, tp, false, false, POS_FACEUP)?;
        let e1 = duel.create_effect(c);
        e1.set_type(duel, EFFECT_TYPE_FIELD);
        e1.set_property(duel, EFFECT_FLAG_PLAYER_TARGET);
        e1.set_range(duel, LOCATION_MZONE);
        e1.set_code(duel, EFFECT_CANNOT_SPECIAL_SUMMON);
        e1.set_absolute_range(duel, tp, 1, 0);
        e1.set_target_card(duel, |duel, _, c| {
            c.is_location(duel, LOCATION_EXTRA) && !c.is_type(duel, TYPE_SYNCHRO)
        });
        e1.set_reset(duel, RESET_EVENT + RESETS_STANDARD);
        token.register_effect_full(duel, e1, true);
        let e2 = aux::create_continuous_lizard_check(duel, c, LOCATION_MZONE, |duel, _, c| !c.is_original_type(duel, TYPE_SYNCHRO));
        e2.set_reset(duel, RESET_EVENT + RESETS_STANDARD);
        token.register_effect_full(duel, e2, true);
    }
    Ok(())
}

fn spcostfilter(c: CardId, duel: &Duel) -> bool {
    (c.is_set_card(duel, &[SET_SWORDSOUL]) || (c.is_monster(duel) && c.is_race(duel, RACE_WYRM))) && c.is_discardable(duel)
}

fn spcost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let (c, tp) = (a.e.get_handler(duel), a.tp);
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(spcostfilter, tp, LOCATION_HAND, 0, 1, Some(c.into())));
    }
    duel.discard_hand_full(tp, spcostfilter, 1, 1, REASON_COST | REASON_DISCARD, Some(c.into()))?;
    Ok(false)
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (c, tp) = (a.e.get_handler(duel), a.tp);
    if chk == 0 {
        return Ok(duel.get_location_count(tp, LOCATION_MZONE) > 0 && c.is_can_be_special_summoned(duel, a.e, 0, tp, false, false));
    }
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, Some(c.into()), 1, 0, 0);
    Ok(false)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (c, tp) = (a.e.get_handler(duel), a.tp);
    if c.is_relate_to_effect(duel, a.e)
        && duel.special_summon(c, 1, tp, tp, false, false, POS_FACEUP)? > 0
        && duel.get_location_count(tp, LOCATION_MZONE) > 0
        && duel.is_player_can_special_summon_monster(tp, TOKEN_SWORDSOUL, token_stats())
        && duel.select_yes_no(tp, aux::stringid(ID, 2))?
    {
        duel.break_effect()?;
        summon_token(duel, c, tp)?;
        duel.special_summon_complete()?;
    }
    Ok(())
}

fn damtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(true);
    }
    duel.set_target_player(1 - a.tp);
    duel.set_target_param(1200);
    duel.set_operation_info(0, CATEGORY_DAMAGE, None, 0, 1 - a.tp, 1200);
    Ok(false)
}

fn damop(duel: &mut Duel, _: &Args) -> Script<()> {
    let p: u8 = duel.get_chain_info(0, CHAININFO_TARGET_PLAYER);
    let d: u32 = duel.get_chain_info(0, CHAININFO_TARGET_PARAM);
    duel.damage(p, d, REASON_EFFECT)?;
    Ok(())
}
