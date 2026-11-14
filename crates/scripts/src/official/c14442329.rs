// JJ "Kewl Tune"
use crate::prelude::*;
use crate::testing::{Check, DECK, HAND, MZONE};

const ID: u32 = 14442329;

const KEWL_TUNE_LOUDNESS_WAR: u32 = 41069676;

pub const LISTED_NAMES: &[u32] = &[KEWL_TUNE_LOUDNESS_WAR];
pub const LISTED_SERIES: &[u16] = &[SET_KEWL_TUNE];

// In hand, with Kewl Tune Cue on the field to release and Kewl Tune Mix and Clip in the Deck to add or summon.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, HAND),
    (16387555, MZONE),
    (16387555, MZONE),
    (16509007, DECK),
    (43904702, DECK),
    (16509007, DECK),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 1))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e0 = duel.create_effect(c);
    e0.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e0.set_code(duel, EVENT_FREE_CHAIN);
    c.register_effect(duel, e0);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_range(duel, LOCATION_FZONE);
    e1.set_target_range(duel, LOCATION_HAND | LOCATION_MZONE, 0);
    e1.set_code(duel, EFFECT_EXTRA_SUMMON_COUNT);
    e1.set_target_card(duel, |duel, _, c| c.is_type(duel, TYPE_TUNER));
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_FIELD);
    e2.set_code(duel, EFFECT_UPDATE_ATTACK);
    e2.set_range(duel, LOCATION_FZONE);
    e2.set_target_range(duel, LOCATION_MZONE, 0);
    e2.set_target_card(duel, |duel, _, c| c.is_code(duel, KEWL_TUNE_LOUDNESS_WAR));
    e2.set_value(duel, 3300);
    e2.set_condition(duel, |duel, a| {
        let tuner = aux::faceup_filter(|c, duel| c.is_type(duel, TYPE_TUNER));
        duel.is_existing_matching_card(tuner, a.e.get_handler_player(duel), 0, LOCATION_MZONE | LOCATION_GRAVE, 1, None)
    });
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 1));
    e3.set_category(duel, CATEGORY_TOHAND + CATEGORY_SEARCH + CATEGORY_SPECIAL_SUMMON);
    e3.set_type(duel, EFFECT_TYPE_IGNITION);
    e3.set_range(duel, LOCATION_FZONE);
    e3.set_count_limit(duel, 1, ID);
    e3.set_cost(duel, thspcost);
    e3.set_target(duel, thsptg);
    e3.set_operation(duel, thspop);
    c.register_effect(duel, e3);
}

fn costfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    let mmz_chk = duel.get_mzone_count(tp, Some(c.into())) > 0;
    c.is_type(duel, TYPE_TUNER)
        && duel.is_existing_matching_card(|sc, duel| thspfilter(sc, duel, e, tp, mmz_chk), tp, LOCATION_DECK, 0, 1, None)
}

fn thspfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8, mmz_chk: bool) -> bool {
    c.is_set_card(duel, &[SET_KEWL_TUNE])
        && c.is_monster(duel)
        && (c.is_able_to_hand(duel) || (mmz_chk && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)))
}

fn thspcost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let filter = |c, duel: &Duel| costfilter(c, duel, e, tp);
    if chk == 0 {
        return Ok(duel.check_release_group_cost(tp, filter, (1, 1), false, None, None));
    }
    let g = duel.select_release_group_cost(tp, filter, (1, 1), false, None, None)?;
    duel.release(g, REASON_COST)?;
    Ok(false)
}

fn thsptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(true);
    }
    duel.set_possible_operation_info(0, CATEGORY_TOHAND, None, 1, a.tp, LOCATION_DECK as i32);
    duel.set_possible_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, a.tp, LOCATION_DECK as i32);
    Ok(false)
}

fn thspop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let mmz_chk = duel.get_location_count(tp, LOCATION_MZONE) > 0;
    duel.hint(HINT_SELECTMSG, tp, aux::stringid(ID, 2));
    let filter = |c, duel: &Duel| thspfilter(c, duel, e, tp, mmz_chk);
    let sc = duel.select_matching_card(tp, filter, tp, LOCATION_DECK, 0, 1, 1, None)?.get_first();
    if let Some(sc) = sc {
        aux::to_hand_or_else(
            duel,
            sc,
            tp,
            |sc, duel| mmz_chk && sc.is_can_be_special_summoned(duel, e, 0, tp, false, false),
            |duel| duel.special_summon(sc, 0, tp, tp, false, false, POS_FACEUP).map(drop),
            aux::stringid(ID, 3),
        )?;
    }
    register_tuner_limit(duel, e, tp, aux::stringid(ID, 4));
    Ok(())
}

// The rest of this turn's "you cannot Special Summon, except Tuners", shared with Kewl Tune Playlist.
pub fn register_tuner_limit(duel: &mut Duel, e: EffectId, tp: u8, description: u64) {
    let e1 = duel.create_effect(e.get_handler(duel));
    e1.set_description(duel, description);
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_property(duel, EFFECT_FLAG_PLAYER_TARGET + EFFECT_FLAG_CLIENT_HINT);
    e1.set_code(duel, EFFECT_CANNOT_SPECIAL_SUMMON);
    e1.set_target_range(duel, 1, 0);
    e1.set_target_card(duel, super::c16387555::splimit);
    e1.set_reset(duel, RESET_PHASE | PHASE_END);
    duel.register_effect(e1, tp);
}
