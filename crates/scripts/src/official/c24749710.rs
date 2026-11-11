// Mind Shuffle
use crate::prelude::*;
use crate::testing::{Check, DECK, HAND, MZONE, OPPONENT, SZONE};

const ID: u32 = 24749710;

pub const LISTED_NAMES: &[u32] = &[CARD_LIGHT_AND_DARKNESS_RITUAL];
// A copy Set and one in the hand, Black Luster Soldier - Soldier of Light and Darkness in the field and Deck and Magician of
// Dark Chaos - Black Chaos in the hand, with Spells of the opponent's to respond to.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, SZONE),
    (70405001, MZONE),
    (70405001, DECK),
    (44001993, HAND),
    (53129443, HAND | OPPONENT),
    (12580477, HAND | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e0 = duel.create_effect(c);
    e0.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e0.set_code(duel, EVENT_FREE_CHAIN);
    c.register_effect(duel, e0);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_TOHAND + CATEGORY_SEARCH + CATEGORY_HANDES);
    e1.set_type(duel, EFFECT_TYPE_QUICK_O);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_range(duel, LOCATION_SZONE);
    e1.set_count_limit(duel, 1, ID);
    e1.set_cost(duel, |duel, a, chk| {
        Cost::and(duel, a, chk, &[opccost1, Cost::hint_selected_effect])
    });
    e1.set_target(duel, thtg);
    e1.set_operation(duel, thop);
    e1.set_hint_timing(duel, 0, TIMING_STANDBY_PHASE | TIMING_MAIN_END | TIMINGS_CHECK_MONSTER_E);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e2.set_type(duel, EFFECT_TYPE_QUICK_O);
    e2.set_code(duel, EVENT_CHAINING);
    e2.set_range(duel, LOCATION_SZONE);
    e2.set_condition(duel, |_, a| a.rp == 1 - a.tp);
    e2.set_cost(duel, |duel, a, chk| {
        Cost::and(duel, a, chk, &[spcost, opccost2, Cost::hint_selected_effect])
    });
    e2.set_target(duel, sptg);
    e2.set_operation(duel, spop);
    c.register_effect(duel, e2);
}

fn opccost1(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    if chk == 0 {
        return Ok(!duel.has_flag_effect(a.tp, ID));
    }
    duel.register_flag_effect(a.tp, ID + 1, RESET_CHAIN, 0, 1);
    Ok(false)
}

fn opccost2(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    if chk == 0 {
        return Ok(!duel.has_flag_effect(a.tp, ID + 1));
    }
    duel.register_flag_effect(a.tp, ID, RESET_CHAIN, 0, 1);
    Ok(false)
}

fn thfilter(c: CardId, duel: &Duel) -> bool {
    c.is_monster(duel) && c.lists_code(duel, CARD_LIGHT_AND_DARKNESS_RITUAL) && c.is_able_to_hand(duel)
}

fn thtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    stock::exists_target(duel, a, chk, thfilter, LOCATION_DECK, CATEGORY_TOHAND)
}

fn thop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_ATOHAND);
    let g = duel.select_matching_card(tp, thfilter, tp, LOCATION_DECK, 0, 1, 1, None)?;
    if !g.is_empty() && duel.send_to_hand(g, PLAYER_NONE, REASON_EFFECT)? > 0 {
        duel.confirm_cards(1 - tp, g)?;
        duel.shuffle_hand(tp)?;
        duel.break_effect()?;
        duel.discard_hand(tp, aux::TRUE, 1, 1, REASON_EFFECT | REASON_DISCARD)?;
    }
    Ok(())
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8, code: u32) -> bool {
    c.is_monster(duel)
        && c.lists_code(duel, CARD_LIGHT_AND_DARKNESS_RITUAL)
        && !c.is_code(duel, code)
        && c.is_can_be_special_summoned(duel, e, 0, tp, true, false)
}

fn spcostfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_level_above(duel, 7)
        && c.is_faceup(duel)
        && c.is_able_to_hand_as_cost(duel)
        && duel.get_mzone_count(tp, Some(c.into())) > 0
        && duel.is_existing_matching_card(
            |sc, duel| spfilter(sc, duel, e, tp, c.get_code(duel)),
            tp,
            LOCATION_HAND,
            0,
            1,
            None,
        )
}

fn spcost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let filter = |c, duel: &Duel| spcostfilter(c, duel, e, tp);
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(filter, tp, LOCATION_MZONE, 0, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_RTOHAND);
    if let Some(sc) = duel
        .select_matching_card(tp, filter, tp, LOCATION_MZONE, 0, 1, 1, None)?
        .get_first()
    {
        duel.hint_selection(sc);
        duel.send_to_hand(sc, PLAYER_NONE, REASON_COST)?;
        let code = sc.get_code(duel);
        e.set_label(duel, i64::from(code));
    }
    Ok(false)
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(true);
    }
    let label = a.e.get_label(duel);
    duel.set_target_param(label as u32);
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, a.tp, LOCATION_HAND as i32);
    Ok(false)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    if duel.get_location_count(tp, LOCATION_MZONE) <= 0 {
        return Ok(());
    }
    let code: u32 = duel.get_chain_info(0, CHAININFO_TARGET_PARAM);
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
    let g = duel.select_matching_card(tp, |c, duel| spfilter(c, duel, e, tp, code), tp, LOCATION_HAND, 0, 1, 1, None)?;
    if !g.is_empty() {
        duel.special_summon(g, 0, tp, tp, true, false, POS_FACEUP)?;
    }
    Ok(())
}
