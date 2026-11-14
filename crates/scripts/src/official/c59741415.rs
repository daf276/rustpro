// Machina Overdrive
use crate::prelude::*;
use crate::testing::{Check, DECK, GRAVE, HAND, MZONE};

const ID: u32 = 59741415;

pub const LISTED_SERIES: &[u16] = &[SET_MACHINA];
// Machina Gearframe on the field to replace with Machina Fortress from the Deck, and Machines in the GY to shuffle.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, GRAVE),
    (42940404, MZONE),
    (42940404, MZONE),
    (5556499, DECK),
    (5556499, DECK),
    (5556499, GRAVE),
    (42940404, GRAVE),
    (46033517, GRAVE),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(0)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON + CATEGORY_DESTROY);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e1.set_hint_timing(duel, 0, TIMING_END_PHASE);
    e1.set_count_limit(duel, 1, ID);
    e1.set_target(duel, target);
    e1.set_operation(duel, operation);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_category(duel, CATEGORY_TODECK + CATEGORY_DRAW);
    e2.set_type(duel, EFFECT_TYPE_QUICK_O);
    e2.set_code(duel, EVENT_FREE_CHAIN);
    e2.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e2.set_range(duel, LOCATION_GRAVE);
    e2.set_hint_timing(duel, 0, TIMING_END_PHASE);
    e2.set_count_limit(duel, 1, ID);
    e2.set_cost(duel, Cost::self_banish);
    e2.set_target(duel, drtg);
    e2.set_operation(duel, drop);
    c.register_effect(duel, e2);
}

fn filter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    let code = c.get_code(duel);
    c.is_faceup(duel)
        && c.is_race(duel, RACE_MACHINE)
        && duel.is_existing_matching_card(
            |c, duel| spfilter(c, duel, e, tp, code),
            tp,
            LOCATION_HAND | LOCATION_DECK,
            0,
            1,
            None,
        )
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8, code: u32) -> bool {
    c.is_set_card(duel, &[SET_MACHINA])
        && !c.is_code(duel, code)
        && c.is_can_be_special_summoned_full(duel, e, 0, tp, false, false, POS_FACEUP, tp, 0xff)
}

fn target(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if let Some(chkc) = chkc {
        return Ok(chkc.is_controler(duel, tp) && chkc.is_location(duel, LOCATION_MZONE) && filter(chkc, duel, e, tp));
    }
    if chk == 0 {
        return Ok(duel.get_location_count(tp, LOCATION_MZONE) > 0
            && duel.is_existing_target(|c, duel| filter(c, duel, e, tp), tp, LOCATION_MZONE, 0, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_DESTROY);
    let g = duel.select_target(tp, |c, duel| filter(c, duel, e, tp), tp, LOCATION_MZONE, 0, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_DESTROY, g, 1, 0, 0);
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, tp, (LOCATION_HAND | LOCATION_DECK) as i32);
    Ok(false)
}

fn operation(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    if duel.get_location_count(tp, LOCATION_MZONE) <= 0 {
        return Ok(());
    }
    let Some(tc) = duel.get_first_target() else { return Ok(()) };
    if !tc.is_relate_to_effect(duel, e) || !tc.is_faceup(duel) {
        return Ok(());
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
    let code = tc.get_code(duel);
    let g = duel.select_matching_card(
        tp,
        |c, duel| spfilter(c, duel, e, tp, code),
        tp,
        LOCATION_HAND | LOCATION_DECK,
        0,
        1,
        1,
        None,
    )?;
    if !g.is_empty() && duel.special_summon(g, 0, tp, tp, false, false, POS_FACEUP)? != 0 {
        duel.destroy(tc, REASON_EFFECT)?;
    }
    Ok(())
}

fn tdfilter(c: CardId, duel: &Duel) -> bool {
    (c.is_faceup(duel) || c.is_location(duel, LOCATION_GRAVE))
        && c.is_monster(duel)
        && c.is_race(duel, RACE_MACHINE)
        && c.is_able_to_deck(duel)
}

fn drtg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    let loc = LOCATION_REMOVED | LOCATION_GRAVE;
    if let Some(chkc) = chkc {
        return Ok((chkc.is_location(duel, LOCATION_REMOVED) || chkc.is_location(duel, LOCATION_GRAVE))
            && chkc.is_controler(duel, tp)
            && tdfilter(chkc, duel));
    }
    if chk == 0 {
        return Ok(duel.is_player_can_draw(tp, 1) && duel.is_existing_target(tdfilter, tp, loc, 0, 3, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TODECK);
    let g = duel.select_target(tp, tdfilter, tp, loc, 0, 3, 3, None)?;
    duel.set_operation_info(0, CATEGORY_TODECK, g, g.map_or(0, |g| g.len() as u8), 0, 0);
    duel.set_operation_info(0, CATEGORY_DRAW, None, 0, tp, 1);
    Ok(false)
}

fn drop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    let tg = duel
        .get_chain_info::<Option<CardSet>>(0, CHAININFO_TARGET_CARDS)
        .unwrap_or_default()
        .filter(duel, |c, duel| c.is_relate_to_effect(duel, a.e), None);
    if tg.is_empty() {
        return Ok(());
    }
    duel.send_to_deck(tg, PLAYER_NONE, SEQ_DECKTOP, REASON_EFFECT)?;
    let g = duel.get_operated_group();
    if g.is_exists(duel, |c, duel| c.is_location(duel, LOCATION_DECK), 1, None) {
        duel.shuffle_deck(tp)?;
    }
    let ct = g.filter_count(duel, |c, duel| c.is_location(duel, LOCATION_DECK | LOCATION_EXTRA), None);
    if ct > 0 {
        duel.break_effect()?;
        duel.draw(tp, 1, REASON_EFFECT)?;
    }
    Ok(())
}
