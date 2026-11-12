// Alien Shocktrooper M-Frame
use crate::prelude::*;
use crate::testing::{Check, GRAVE, HAND, MZONE, OPPONENT};

const ID: u32 = 74974229;
const DISCARDED_LEVEL: u8 = 0;

pub const COUNTER_PLACE_LIST: &[u16] = &[COUNTER_A];

struct Procedure;

impl LinkProcedure for Procedure {
    const MIN: usize = 2;
    const MAX: Option<usize> = Some(2);

    fn f(c: CardId, duel: &Duel, lc: CardId, sumtype: u64, tp: u8) -> bool {
        c.is_race_full(duel, RACE_REPTILE, Some(lc), sumtype, tp)
    }
}

// On the field with Reptiles in the hand to discard and the Graveyard to Special Summon, the opponent's Raigeki to destroy it and monsters
// of theirs to hold A-Counters.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (ID, MZONE),
    (82742611, HAND),
    (94412545, HAND),
    (16353197, GRAVE),
    (17115745, GRAVE),
    (62327910, MZONE | OPPONENT),
    (30655537, MZONE | OPPONENT),
    (12580477, HAND | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Link::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_COUNTER);
    e1.set_type(duel, EFFECT_TYPE_QUICK_O);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_count_limit(duel, 1, ID);
    e1.set_hint_timing(duel, 0, TIMINGS_CHECK_MONSTER | TIMING_END_PHASE);
    e1.set_cost(duel, cost);
    e1.set_target(duel, target);
    e1.set_operation(duel, operation);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e2.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e2.set_property(duel, EFFECT_FLAG_DELAY);
    e2.set_code(duel, EVENT_TO_GRAVE);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_condition(duel, spcon);
    e2.set_target(duel, sptg);
    e2.set_operation(duel, spop);
    c.register_effect(duel, e2);
}

fn costfilter(c: CardId, duel: &Duel) -> bool {
    c.is_monster(duel) && c.is_discardable(duel) && c.get_original_level(duel) > 0
}

fn cost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(costfilter, tp, LOCATION_HAND, 0, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_DISCARD);
    let g = duel.select_matching_card(tp, costfilter, tp, LOCATION_HAND, 0, 1, 1, None)?;
    duel.send_to_grave(g, REASON_COST | REASON_DISCARD)?;
    let level = g.get_first().map_or(0, |tc| i64::from(tc.get_original_level(duel)));
    e.set_chain_data(duel, DISCARDED_LEVEL, level);
    Ok(false)
}

fn target(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    Ok(chk == 0 && duel.is_existing_matching_card(CardId::is_faceup, a.tp, LOCATION_MZONE, LOCATION_MZONE, 1, None))
}

fn operation(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let g = duel.get_matching_group(CardId::is_faceup, tp, LOCATION_MZONE, LOCATION_MZONE, None);
    if g.is_empty() {
        return Ok(());
    }
    let level = e.get_chain_data(duel, DISCARDED_LEVEL).unwrap_or(0);
    if let (1, Some(tc)) = (g.len(), g.get_first()) {
        tc.add_counter(duel, COUNTER_A, level as u16);
        return Ok(());
    }
    for _ in 0..level {
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_COUNTER);
        if let Some(tc) = g.select(duel, tp, 1, 1, None)?.get_first() {
            tc.add_counter(duel, COUNTER_A, 1);
        }
    }
    Ok(())
}

fn spcon(duel: &Duel, a: &Args) -> bool {
    let c = a.e.get_handler(duel);
    c.is_reason(duel, REASON_DESTROY) && c.is_reason(duel, REASON_BATTLE | REASON_EFFECT)
}

fn acfilter(c: CardId, duel: &Duel) -> bool {
    c.get_counter(duel, COUNTER_A) > 0
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_race(duel, RACE_REPTILE) && !c.is_link_monster(duel) && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if chk == 0 {
        return Ok(duel.get_location_count(tp, LOCATION_MZONE) > 0
            && duel.is_existing_matching_card(acfilter, tp, 0, LOCATION_MZONE, 1, None)
            && duel.is_existing_matching_card(|c, duel| spfilter(c, duel, e, tp), tp, LOCATION_GRAVE, 0, 1, None));
    }
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, tp, LOCATION_GRAVE as i32);
    Ok(false)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let ft = duel.get_location_count(tp, LOCATION_MZONE);
    if ft == 0 {
        return Ok(());
    }
    let ct = duel.get_matching_group_count(acfilter, tp, 0, LOCATION_MZONE, None) as i32;
    if ct == 0 {
        return Ok(());
    }
    let g = duel.get_matching_group(
        aux::necro_valley_filter(|c, duel| spfilter(c, duel, e, tp)),
        tp,
        LOCATION_GRAVE,
        0,
        None,
    );
    if g.is_empty() {
        return Ok(());
    }
    let ct = if duel.is_player_affected_by_effect(tp, CARD_BLUEEYES_SPIRIT).is_some() {
        1
    } else {
        ft.min(ct)
    };
    let sg = aux::select_unselect_group(duel, g, 1, ct as usize, &aux::dncheck, tp, HINTMSG_SPSUMMON)?;
    if !sg.is_empty() {
        duel.special_summon(sg, 0, tp, tp, false, false, POS_FACEUP)?;
    }
    Ok(())
}
