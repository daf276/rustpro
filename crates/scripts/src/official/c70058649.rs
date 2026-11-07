// Clown Crew Soiree Operations
use super::c83232904::{register_activation_limit, special_summon_clown_crew, spfilter};
use crate::prelude::*;
use crate::testing::{Check, DECK, EXTRA, GRAVE, HAND, MZONE};

const ID: u32 = 70058649;

// In the hand and the Graveyard, with Clown Crew monsters in the Deck (to search or Special Summon), the Extra Deck and the hand,
// and Clown Crew Flair on the field to Tribute for the one in the hand.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, GRAVE),
    (82159583, DECK),
    (42759961, DECK),
    (31533473, EXTRA),
    (82159583, HAND),
    (42759961, MZONE),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
    Check::SpecialSummonedCode(82159583),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_cost(duel, effcost);
    e1.set_target(duel, efftg);
    e1.set_operation(duel, effop);
    e1.set_hint_timing(duel, 0, TIMING_STANDBY_PHASE | TIMING_MAIN_END | TIMINGS_CHECK_MONSTER_E);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_SUMMON);
    e2.set_type(duel, EFFECT_TYPE_QUICK_O);
    e2.set_code(duel, EVENT_FREE_CHAIN);
    e2.set_range(duel, LOCATION_GRAVE);
    e2.set_condition(duel, |duel, _| duel.is_main_phase_full(None));
    e2.set_cost(duel, Cost::self_banish);
    e2.set_target(duel, tribsumtg);
    e2.set_operation(duel, tribsumop);
    e2.set_hint_timing(duel, 0, TIMING_MAIN_END | TIMINGS_CHECK_MONSTER);
    c.register_effect(duel, e2);
}

fn thfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_CLOWN_CREW]) && c.is_monster(duel) && c.is_able_to_hand(duel)
}

fn can_special_summon(duel: &Duel, e: EffectId, tp: u8) -> bool {
    let mmz_chk = duel.get_location_count(tp, LOCATION_MZONE) > 0;
    duel.is_existing_matching_card(
        |c, duel| spfilter(c, duel, e, tp, mmz_chk),
        tp,
        LOCATION_DECK | LOCATION_EXTRA,
        0,
        1,
        None,
    )
}

fn effcost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    e.set_label(duel, -100);
    let b1 = !duel.has_flag_effect(tp, ID) && can_special_summon(duel, e, tp);
    let b2 = !duel.has_flag_effect(tp, ID + 1) && duel.is_existing_matching_card(thfilter, tp, LOCATION_DECK, 0, 1, None);
    Ok(chk == 0 && (b1 || b2))
}

fn efftg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let cost_skip = e.get_label(duel) != -100;
    let b1 = (cost_skip || !duel.has_flag_effect(tp, ID)) && can_special_summon(duel, e, tp);
    let b2 = (cost_skip || !duel.has_flag_effect(tp, ID + 1)) && duel.is_existing_matching_card(thfilter, tp, LOCATION_DECK, 0, 1, None);
    if chk == 0 {
        e.set_label(duel, 0);
        return Ok(b1 || b2);
    }
    let op = duel.select_effect(tp, &[(b1, aux::stringid(ID, 2)), (b2, aux::stringid(ID, 3))])?;
    e.set_label(duel, op.map_or(0, |op| op as i64));
    if op == Some(1) {
        e.set_category(duel, CATEGORY_SPECIAL_SUMMON);
        if !cost_skip {
            duel.register_flag_effect(tp, ID, RESET_PHASE | PHASE_END, 0, 1);
        }
        duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, tp, (LOCATION_DECK | LOCATION_EXTRA) as i32);
    } else if op == Some(2) {
        e.set_category(duel, CATEGORY_TOHAND + CATEGORY_SEARCH);
        if !cost_skip {
            duel.register_flag_effect(tp, ID + 1, RESET_PHASE | PHASE_END, 0, 1);
        }
        duel.set_operation_info(0, CATEGORY_TOHAND, None, 1, tp, LOCATION_DECK as i32);
    }
    Ok(false)
}

fn effop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    if e.is_has_type(duel, EFFECT_TYPE_ACTIVATE) {
        register_activation_limit(duel, e, tp, aux::stringid(ID, 4));
    }
    match e.get_label(duel) {
        1 => special_summon_clown_crew(duel, e, tp)?,
        2 => {
            duel.hint(HINT_SELECTMSG, tp, HINTMSG_ATOHAND);
            let g = duel.select_matching_card(tp, thfilter, tp, LOCATION_DECK, 0, 1, 1, None)?;
            if !g.is_empty() {
                duel.send_to_hand(g, PLAYER_NONE, REASON_EFFECT)?;
                duel.confirm_cards(1 - tp, g)?;
            }
        }
        _ => {}
    }
    Ok(())
}

fn tribsumfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_CLOWN_CREW]) && c.is_summonable_full(duel, true, None, 1, 0x1f)
}

fn tribsumtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(tribsumfilter, tp, LOCATION_HAND, 0, 1, None));
    }
    duel.set_operation_info(0, CATEGORY_SUMMON, None, 1, tp, LOCATION_HAND as i32);
    Ok(false)
}

fn tribsumop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SUMMON);
    let g = duel.select_matching_card(tp, tribsumfilter, tp, LOCATION_HAND, 0, 1, 1, None)?;
    if let Some(sc) = g.get_first() {
        duel.summon_full(tp, sc, true, None, 1, 0x1f)?;
    }
    Ok(())
}
