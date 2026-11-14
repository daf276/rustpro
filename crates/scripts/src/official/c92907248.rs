// Wake Up Centur-Ion!
use crate::prelude::*;
use crate::testing::{Check, DECK, GRAVE, HAND, MZONE};

const ID: u32 = 92907248;
const TOKEN_CENTURION: u32 = ID + 1;

pub const LISTED_SERIES: &[u16] = &[SET_CENTUR_ION];
pub const LISTED_NAMES: &[u32] = &[TOKEN_CENTURION, ID];
// Centur-Ion Trudea on the field to place itself as a Continuous Trap, and in the Deck to send.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, HAND),
    (ID, GRAVE),
    (42493140, MZONE),
    (42493140, MZONE),
    (42493140, DECK),
    (15005145, DECK),
];
pub const CHECKS: &[Check] = &[
    Check::SpecialSummonedCode(TOKEN_CENTURION),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON + CATEGORY_TOKEN);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_hint_timing(duel, 0, TIMING_MAIN_END | TIMINGS_CHECK_MONSTER_E);
    e1.set_count_limit(duel, 1, ID);
    e1.set_condition(duel, |duel, a| {
        duel.is_existing_matching_card(cfilter, a.tp, LOCATION_STZONE, 0, 1, None)
    });
    e1.set_target(duel, tokentg);
    e1.set_operation(duel, tokenop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_TOGRAVE);
    e2.set_type(duel, EFFECT_TYPE_IGNITION);
    e2.set_range(duel, LOCATION_GRAVE);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_cost(duel, Cost::self_banish);
    e2.set_target(duel, tgytg);
    e2.set_operation(duel, tgyop);
    c.register_effect(duel, e2);
}

fn cfilter(c: CardId, duel: &Duel) -> bool {
    c.is_faceup(duel) && c.is_original_type(duel, TYPE_MONSTER)
}

fn can_summon_token(duel: &mut Duel, tp: u8, level: u32) -> bool {
    let stats = MonsterStats::new(SET_CENTUR_ION, TYPES_TOKEN, 0, 0, level, RACE_PYRO, ATTRIBUTE_DARK);
    duel.is_player_can_special_summon_monster_full(tp, TOKEN_CENTURION, stats, POS_FACEUP, tp, 0)
}

fn tokentg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let lv4 = can_summon_token(duel, tp, 4);
    let lv8 = can_summon_token(duel, tp, 8);
    if chk == 0 {
        return Ok((lv4 || lv8) && duel.get_location_count(tp, LOCATION_MZONE) > 0);
    }
    let levels: Vec<u64> = [(lv4, 4), (lv8, 8)]
        .into_iter()
        .filter(|&(can, _)| can)
        .map(|(_, level)| level)
        .collect();
    let lvl = duel.announce_number(tp, &levels)?;
    e.set_label(duel, lvl as i64);
    duel.set_operation_info(0, CATEGORY_TOKEN, None, 1, 0, 0);
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, tp, 0);
    Ok(false)
}

fn tokenop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let lvl = e.get_label(duel) as u32;
    if duel.get_location_count(tp, LOCATION_MZONE) <= 0 || !can_summon_token(duel, tp, lvl) {
        return Ok(());
    }
    let Some(token) = duel.create_token(tp, TOKEN_CENTURION) else {
        return Ok(());
    };
    let e1 = duel.create_effect(e.get_handler(duel));
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE);
    e1.set_code(duel, EFFECT_CHANGE_LEVEL);
    e1.set_value(duel, lvl as i32);
    e1.set_reset(duel, RESET_EVENT | (RESETS_STANDARD & !RESET_TOFIELD));
    token.register_effect(duel, e1);
    if duel.special_summon_step(token, 0, tp, tp, false, false, POS_FACEUP)? {
        let e2 = e1.clone_effect(duel);
        e2.set_description(duel, 3309);
        e2.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE + EFFECT_FLAG_CLIENT_HINT);
        e2.set_code(duel, EFFECT_CANNOT_BE_FUSION_MATERIAL);
        e2.set_value(duel, 1);
        token.register_effect_full(duel, e2, true);
        let e3 = e2.clone_effect(duel);
        e3.set_description(duel, 3312);
        e3.set_code(duel, EFFECT_CANNOT_BE_LINK_MATERIAL);
        token.register_effect_full(duel, e3, true);
    }
    duel.special_summon_complete()?;
    Ok(())
}

fn tgyfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_CENTUR_ION]) && !c.is_code(duel, ID) && c.is_able_to_grave(duel)
}

fn tgytg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    stock::exists_target(duel, a, chk, tgyfilter, LOCATION_DECK, CATEGORY_TOGRAVE)
}

fn tgyop(duel: &mut Duel, a: &Args) -> Script<()> {
    stock::send_to_grave(duel, a, tgyfilter, LOCATION_DECK)
}
