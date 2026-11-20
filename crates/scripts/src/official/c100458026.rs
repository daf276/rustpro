// The Glitter of Raise Moon
use super::c100458016::{draw, draw_target, drcost, track_searches};
use crate::prelude::*;
use crate::testing::{Check, EXTRA, GRAVE, HAND};

const ID: u32 = 100458026;

pub const LISTED_SERIES: &[u16] = &[SET_RAISE_MOON];

// In the hand with the Raise Moon Xyz Monsters in the Extra Deck and Raise Moon cards to shuffle in, and in the Graveyard.
pub const SUPPORT: &[(u32, u16)] = &[(ID, HAND), (ID, GRAVE), (100458022, EXTRA), (100458023, EXTRA), (100458017, GRAVE)];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON + CATEGORY_TODECK);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_cost(duel, drcost::<ID, 2, 0>);
    e1.set_target(duel, sptg);
    e1.set_operation(duel, spop);
    e1.set_hint_timing(duel, 0, TIMING_STANDBY_PHASE | TIMING_MAIN_END | TIMINGS_CHECK_MONSTER_E);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_DRAW);
    e2.set_type(duel, EFFECT_TYPE_QUICK_O);
    e2.set_code(duel, EVENT_FREE_CHAIN);
    e2.set_range(duel, LOCATION_GRAVE);
    e2.set_count_limit(duel, 1, ID);
    e2.set_cost(duel, |duel, a, chk| {
        Cost::and(duel, a, chk, &[Cost::self_banish, drcost::<ID, 2, 0>])
    });
    e2.set_target(duel, draw_target);
    e2.set_operation(duel, draw);
    e2.set_hint_timing(duel, 0, TIMING_STANDBY_PHASE | TIMING_MAIN_END | TIMINGS_CHECK_MONSTER_E);
    c.register_effect(duel, e2);
    track_searches::<ID, 0>(duel, c);
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_set_card(duel, &[SET_RAISE_MOON])
        && c.is_xyz_monster(duel)
        && duel.get_location_count_from_ex(tp, tp, None, c) > 0
        && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if chk == 0 {
        return Ok(e.is_has_type(duel, EFFECT_TYPE_ACTIVATE)
            && duel.is_existing_matching_card(|c, duel| spfilter(c, duel, e, tp), tp, LOCATION_EXTRA, 0, 1, None));
    }
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, tp, LOCATION_EXTRA as i32);
    duel.set_possible_operation_info(0, CATEGORY_TODECK, None, 1, tp, LOCATION_GRAVE as i32);
    Ok(false)
}

fn tdfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_RAISE_MOON]) && c.is_able_to_deck(duel)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
    let g = duel.select_matching_card(tp, |c, duel| spfilter(c, duel, e, tp), tp, LOCATION_EXTRA, 0, 1, 1, None)?;
    let Some(sc) = g.get_first() else { return Ok(()) };
    if duel.special_summon(sc, 0, tp, tp, false, false, POS_FACEUP)? > 0 && c.is_relate_to_effect(duel, e) {
        c.cancel_to_grave(duel);
        duel.overlay(sc, c)?;
        if duel.is_existing_matching_card(aux::necro_valley_filter(tdfilter), tp, LOCATION_GRAVE, 0, 1, None)
            && duel.select_yes_no(tp, aux::stringid(ID, 3))?
        {
            let g = duel.get_matching_group(aux::necro_valley_filter(tdfilter), tp, LOCATION_GRAVE, 0, None);
            duel.break_effect()?;
            duel.send_to_deck(g, PLAYER_NONE, SEQ_DECKSHUFFLE, REASON_EFFECT)?;
        }
    }
    Ok(())
}
