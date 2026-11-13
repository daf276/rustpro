// Twin Pillars of Ice
use crate::prelude::*;
use crate::testing::{Check, GRAVE, HAND};

const ID: u32 = 88477149;
const TOKEN_ICE_PILLAR: u32 = ID + 1;

pub const LISTED_NAMES: &[u32] = &[TOKEN_ICE_PILLAR];

// In the hand and Graveyard, with WATER Normal Monsters (Aqua Madoor, Change Slime) in the hand to Normal Summon.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, GRAVE),
    (ID, GRAVE),
    (85639257, HAND),
    (85639257, HAND),
    (18914778, HAND),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
    Check::SpecialSummonedCode(TOKEN_ICE_PILLAR),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON + CATEGORY_TOKEN);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_count_limit(duel, 1, ID);
    e1.set_hint_timing(
        duel,
        0,
        TIMING_STANDBY_PHASE | TIMING_MAIN_END | TIMING_BATTLE_START | TIMINGS_CHECK_MONSTER_E,
    );
    e1.set_target(duel, sptg);
    e1.set_operation(duel, spop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_SUMMON);
    e2.set_type(duel, EFFECT_TYPE_IGNITION);
    e2.set_range(duel, LOCATION_GRAVE);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_cost(duel, Cost::self_banish);
    e2.set_target(duel, nstg);
    e2.set_operation(duel, nsop);
    c.register_effect(duel, e2);
}

fn token_check(duel: &mut Duel, tp: u8) -> bool {
    let stats = MonsterStats::new(0, TYPES_TOKEN, 1900, 1200, 4, RACE_AQUA, ATTRIBUTE_WATER);
    duel.is_player_can_special_summon_monster_full(tp, TOKEN_ICE_PILLAR, stats, POS_FACEUP_DEFENSE, tp, 0)
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.is_player_affected_by_effect(tp, CARD_BLUEEYES_SPIRIT).is_none()
            && duel.get_location_count(tp, LOCATION_MZONE) > 1
            && token_check(duel, tp));
    }
    duel.set_operation_info(0, CATEGORY_TOKEN, None, 2, 0, 0);
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 2, tp, 0);
    Ok(false)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    if duel.is_player_affected_by_effect(tp, CARD_BLUEEYES_SPIRIT).is_none()
        && duel.get_location_count(tp, LOCATION_MZONE) > 1
        && token_check(duel, tp)
    {
        for _ in 0..2 {
            if let Some(token) = duel.create_token(tp, TOKEN_ICE_PILLAR) {
                duel.special_summon_step(token, 0, tp, tp, false, false, POS_FACEUP_DEFENSE)?;
            }
        }
        duel.special_summon_complete()?;
    }
    if !e.is_has_type(duel, EFFECT_TYPE_ACTIVATE) {
        return Ok(());
    }
    let e1 = duel.create_effect(e.get_handler(duel));
    e1.set_description(duel, aux::stringid(ID, 2));
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_property(duel, EFFECT_FLAG_PLAYER_TARGET + EFFECT_FLAG_CLIENT_HINT);
    e1.set_code(duel, EFFECT_CANNOT_SPECIAL_SUMMON);
    e1.set_target_range(duel, 1, 0);
    e1.set_target_card(duel, |duel, _, c| c.is_location(duel, LOCATION_EXTRA));
    e1.set_reset(duel, RESET_PHASE | PHASE_END);
    duel.register_effect(e1, tp);
    Ok(())
}

fn nsfilter(c: CardId, duel: &Duel) -> bool {
    c.is_attribute(duel, ATTRIBUTE_WATER) && c.is_summonable(duel, true, None)
}

fn nstg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    stock::exists_target(duel, a, chk, nsfilter, LOCATION_HAND | LOCATION_MZONE, CATEGORY_SUMMON)
}

fn nsop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SUMMON);
    let g = duel.select_matching_card(tp, nsfilter, tp, LOCATION_HAND | LOCATION_MZONE, 0, 1, 1, None)?;
    if let Some(sc) = g.get_first() {
        duel.summon(tp, sc, true, None)?;
    }
    Ok(())
}
