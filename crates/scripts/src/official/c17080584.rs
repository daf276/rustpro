// Abyssrhine, the Atlantean Spirit
use super::c12975671::limit_extra_deck_to_water;
use crate::prelude::*;
use crate::testing::{Check, DECK, GRAVE, HAND, MZONE};

const ID: u32 = 17080584;

pub const LISTED_NAMES: &[u32] = &[ID];
// In the hand (and the Graveyard to draw), with Atlantean Pikeman, a WATER Normal Monster, on the field to Tribute and in
// the hand to discard, and Ignihorse Dragonis, a Level 7 Sea Serpent Normal Monster, in the Deck.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, GRAVE),
    (26976414, MZONE),
    (26976414, HAND),
    (28103028, DECK),
    (28103028, DECK),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_TOHAND + CATEGORY_SEARCH + CATEGORY_SPECIAL_SUMMON);
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_range(duel, LOCATION_HAND);
    e1.set_count_limit(duel, 1, ID);
    e1.set_cost(duel, thspcost);
    e1.set_target(duel, thsptg);
    e1.set_operation(duel, thspop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_DRAW);
    e2.set_type(duel, EFFECT_TYPE_QUICK_O);
    e2.set_code(duel, EVENT_FREE_CHAIN);
    e2.set_range(duel, LOCATION_GRAVE);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_hint_timing(duel, 0, TIMING_STANDBY_PHASE | TIMING_MAIN_END | TIMINGS_CHECK_MONSTER_E);
    e2.set_condition(duel, |duel, a| duel.is_turn_player(1 - a.tp));
    e2.set_cost(duel, drawcost);
    e2.set_target(duel, drawtg);
    e2.set_operation(duel, drawop);
    c.register_effect(duel, e2);
}

fn thspfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8, mmz_check: bool) -> bool {
    c.is_level(duel, &[7])
        && c.is_race(duel, RACE_FISH | RACE_SEASERPENT | RACE_AQUA)
        && (c.is_able_to_hand(duel) || (mmz_check && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)))
}

fn thspcostfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_set_card(duel, &[SET_ATLANTEAN, SET_MERMAIL])
        && (c.is_faceup(duel) || c.is_controler(duel, tp))
        && duel.is_existing_matching_card(
            |sc, duel| thspfilter(sc, duel, e, tp, duel.get_mzone_count(tp, Some(c.into())) > 0),
            tp,
            LOCATION_DECK,
            0,
            1,
            None,
        )
}

fn thspcost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    e.set_label(duel, 1);
    let c = e.get_handler(duel);
    let filter = |sc, duel: &Duel| thspcostfilter(sc, duel, e, tp);
    if chk == 0 {
        return Ok(c.is_releasable(duel) && duel.check_release_group_cost(tp, filter, (1, 1), true, None, Some(c.into())));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_RELEASE);
    let mut rg = duel.select_release_group_cost(tp, filter, (1, 1), true, None, Some(c.into()))?;
    rg.insert(c);
    duel.release(rg, REASON_COST)?;
    Ok(false)
}

fn thsptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if chk == 0 {
        let cost_chk = e.get_label(duel) == 1 || duel.get_location_count(tp, LOCATION_MZONE) > 0;
        e.set_label(duel, 0);
        return Ok(duel.is_existing_matching_card(|c, duel| thspfilter(c, duel, e, tp, cost_chk), tp, LOCATION_DECK, 0, 1, None));
    }
    duel.set_possible_operation_info(0, CATEGORY_TOHAND, None, 1, tp, LOCATION_DECK as i32);
    duel.set_possible_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, tp, LOCATION_DECK as i32);
    Ok(false)
}

fn thspop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let mmz_check = duel.get_location_count(tp, LOCATION_MZONE) > 0;
    duel.hint(HINT_SELECTMSG, tp, aux::stringid(ID, 2));
    let g = duel.select_matching_card(
        tp,
        |c, duel| thspfilter(c, duel, e, tp, mmz_check),
        tp,
        LOCATION_DECK,
        0,
        1,
        1,
        None,
    )?;
    if let Some(sc) = g.get_first() {
        aux::to_hand_or_else(
            duel,
            sc,
            tp,
            |sc, duel| mmz_check && sc.is_can_be_special_summoned(duel, e, 0, tp, false, false),
            |duel| duel.special_summon(sc, 0, tp, tp, false, false, POS_FACEUP).map(drop),
            aux::stringid(ID, 3),
        )?;
    }
    limit_extra_deck_to_water(duel, e.get_handler(duel), tp, aux::stringid(ID, 4));
    Ok(())
}

fn discardable(c: CardId, duel: &Duel) -> bool {
    c.is_discardable(duel) && c.is_able_to_grave_as_cost(duel)
}

fn drawcost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let tp = a.tp;
    let c = a.e.get_handler(duel);
    if chk == 0 {
        return Ok(c.is_able_to_remove_as_cost(duel) && duel.is_existing_matching_card(discardable, tp, LOCATION_HAND, 0, 1, None));
    }
    duel.remove(c, POS_FACEUP, REASON_COST)?;
    duel.discard_hand(tp, discardable, 1, 1, REASON_COST | REASON_DISCARD)?;
    Ok(false)
}

fn drawtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.is_player_can_draw(tp, 1));
    }
    duel.set_target_player(tp);
    duel.set_target_param(1);
    duel.set_operation_info(0, CATEGORY_DRAW, None, 0, tp, 1);
    Ok(false)
}

fn drawop(duel: &mut Duel, _: &Args) -> Script<()> {
    let p: u8 = duel.get_chain_info(0, CHAININFO_TARGET_PLAYER);
    let d: u32 = duel.get_chain_info(0, CHAININFO_TARGET_PARAM);
    duel.draw(p, d, REASON_EFFECT)?;
    Ok(())
}
