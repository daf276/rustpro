// Ogdoadic Daybreak
use crate::prelude::*;
use crate::testing::{Check, DECK, GRAVE, HAND, MZONE, REMOVED};

const ID: u32 = 78789356;
const TOKEN: u32 = 23837055;

pub const LISTED_NAMES: &[u32] = &[TOKEN];
// Keurse (Level 8) on the field to Tribute, a copy in the GY with Nunu banished to shuffle and Nauya in the Deck to send.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, GRAVE),
    (ID, HAND),
    (62383431, MZONE),
    (62383431, MZONE),
    (36010310, REMOVED),
    (36010310, REMOVED),
    (62405028, DECK),
    (62405028, DECK),
];
pub const CHECKS: &[Check] = &[Check::SpecialSummonedCode(TOKEN), Check::ResolvesEffect(aux::stringid(ID, 1))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON + CATEGORY_TOKEN);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_hint_timing(duel, 0, TIMING_END_PHASE);
    e1.set_count_limit(duel, 1, ID);
    e1.set_cost(duel, tkncost);
    e1.set_target(duel, tnktg);
    e1.set_operation(duel, tnkop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_TODECK + CATEGORY_TOGRAVE);
    e2.set_type(duel, EFFECT_TYPE_IGNITION);
    e2.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e2.set_range(duel, LOCATION_GRAVE);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_cost(duel, Cost::self_banish);
    e2.set_target(duel, tdtg);
    e2.set_operation(duel, tdop);
    c.register_effect(duel, e2);
}

fn cfilter(c: CardId, duel: &Duel, tp: u8) -> bool {
    c.is_race(duel, RACE_REPTILE) && c.has_level(duel) && c.is_level_above(duel, 2) && duel.get_mzone_count(tp, Some(c.into())) > 0
}

fn tkncost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    a.e.set_label(duel, -100);
    Ok(chk == 0)
}

fn token_stats() -> MonsterStats {
    MonsterStats::new(SET_OGDOADIC, TYPES_TOKEN, 0, 0, 2, RACE_REPTILE, ATTRIBUTE_DARK)
}

fn tnktg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if chk == 0 {
        if e.get_label(duel) != -100 {
            return Ok(false);
        }
        e.set_label(duel, 0);
        return Ok(
            duel.check_release_group_cost(tp, |c, duel| cfilter(c, duel, tp), (1, 1), false, None, None)
                && duel.is_player_can_special_summon_monster(tp, TOKEN, token_stats()),
        );
    }
    let g = duel.select_release_group_cost(tp, |c, duel| cfilter(c, duel, tp), (1, 1), false, None, None)?;
    let level = g.get_first().map_or(0, |c| c.get_level(duel));
    e.set_label(duel, i64::from(level / 2));
    duel.release(g, REASON_COST)?;
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, tp, 0);
    duel.set_operation_info(0, CATEGORY_TOKEN, None, 1, 0, 0);
    Ok(false)
}

fn tnkop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    if !duel.is_player_can_special_summon_monster(tp, TOKEN, token_stats()) {
        return Ok(());
    }
    let mut ft = duel.get_location_count(tp, LOCATION_MZONE);
    if ft == 0 {
        return Ok(());
    }
    if duel.is_player_affected_by_effect(tp, CARD_BLUEEYES_SPIRIT).is_some() {
        ft = 1;
    }
    let ft = (e.get_label(duel) as i32).min(ft);
    let ct = duel.announce_number_range(tp, 1, ft as u32)?;
    for _ in 0..ct {
        if let Some(token) = duel.create_token(tp, TOKEN) {
            duel.special_summon_step(token, 0, tp, tp, false, false, POS_FACEUP)?;
        }
    }
    duel.special_summon_complete()?;
    Ok(())
}

fn tdfilter(c: CardId, duel: &Duel) -> bool {
    c.is_faceup(duel) && c.is_race(duel, RACE_REPTILE) && c.is_able_to_deck(duel)
}

fn tdtg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_REMOVED) && chkc.is_controler(duel, tp) && tdfilter(chkc, duel));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(tdfilter, tp, LOCATION_REMOVED, 0, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TODECK);
    let g = duel.select_target(tp, tdfilter, tp, LOCATION_REMOVED, 0, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_TODECK, g, 1, tp, 0);
    duel.set_possible_operation_info(0, CATEGORY_TOGRAVE, None, 1, tp, LOCATION_DECK as i32);
    Ok(false)
}

fn tgfilter(c: CardId, duel: &Duel) -> bool {
    c.is_race(duel, RACE_REPTILE) && c.is_able_to_grave(duel)
}

fn tdop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    let Some(tc) = duel.get_first_target() else { return Ok(()) };
    if !(tc.is_relate_to_effect(duel, a.e)
        && duel.send_to_deck(tc, PLAYER_NONE, SEQ_DECKSHUFFLE, REASON_EFFECT)? > 0
        && tc.is_location(duel, LOCATION_DECK | LOCATION_EXTRA))
    {
        return Ok(());
    }
    duel.shuffle_deck(tp)?;
    if duel.is_existing_matching_card(tgfilter, tp, LOCATION_DECK, 0, 1, None) && duel.select_yes_no(tp, aux::stringid(ID, 2))? {
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_TOGRAVE);
        let g = duel.select_matching_card(tp, tgfilter, tp, LOCATION_DECK, 0, 1, 1, None)?;
        if !g.is_empty() {
            duel.break_effect()?;
            duel.send_to_grave(g, REASON_EFFECT)?;
        }
    }
    Ok(())
}
