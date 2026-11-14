// Artmage Peripeteia -Turmoil-
use super::c21720439::{cost, counterfilter, thcon};
use crate::prelude::*;
use crate::testing::{Check, DECK, GRAVE, HAND, MZONE, OPPONENT};

const ID: u32 = 17719582;

pub const LISTED_SERIES: &[u16] = &[SET_ARTMAGE];
// In hand to return Artmage Litera and Special Summon a monster from the opponent's Graveyard, or to discard as the cost of
// Acropolis (in hand to activate, with Artmage monsters in the Deck to add), with a second Acropolis in the Graveyard to add.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, HAND),
    (ID, HAND),
    (97434754, MZONE),
    (26202165, GRAVE | OPPONENT),
    (74733322, HAND),
    (74733322, 0),
    (74733322, 0),
    (74733322, GRAVE),
    (97434754, DECK),
    (34541940, DECK),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_TOHAND + CATEGORY_TOEXTRA + CATEGORY_SPECIAL_SUMMON);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_count_limit(duel, 1, ID);
    e1.set_hint_timing(duel, 0, TIMINGS_CHECK_MONSTER_E + TIMING_MAIN_END + TIMING_STANDBY_PHASE);
    e1.set_cost(duel, cost::<ID>);
    e1.set_target(duel, rthtetg);
    e1.set_operation(duel, rthteop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_TOHAND);
    e2.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e2.set_property(duel, EFFECT_FLAG_DELAY);
    e2.set_code(duel, EVENT_TO_GRAVE);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_condition(duel, thcon);
    e2.set_cost(duel, cost::<ID>);
    e2.set_target(duel, thtg);
    e2.set_operation(duel, thop);
    c.register_effect(duel, e2);
    duel.add_custom_activity_counter(ID, ACTIVITY_SPSUMMON, CounterFilter::Card(counterfilter));
}

fn rthtefilter(c: CardId, duel: &Duel, tp: Option<u8>) -> bool {
    c.is_set_card(duel, &[SET_ARTMAGE])
        && (c.is_able_to_hand(duel) || c.is_able_to_extra(duel))
        && c.is_faceup(duel)
        && tp.is_none_or(|tp| duel.get_mzone_count(tp, Some(c.into())) > 0)
}

fn rthtetg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let spfilter = |c: CardId, duel: &Duel| c.is_can_be_special_summoned(duel, e, 0, tp, false, false);
    if chk == 0 {
        return Ok(duel.is_existing_target(spfilter, tp, 0, LOCATION_GRAVE, 1, None)
            && duel.is_existing_matching_card(|c, duel| rthtefilter(c, duel, Some(tp)), tp, LOCATION_MZONE, 0, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
    let g = duel.select_target(tp, spfilter, tp, 0, LOCATION_GRAVE, 1, 1, None)?;
    duel.set_possible_operation_info(0, CATEGORY_TOHAND, None, 1, tp, LOCATION_MZONE as i32);
    duel.set_possible_operation_info(0, CATEGORY_TOEXTRA, None, 1, tp, LOCATION_MZONE as i32);
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, g, 1, tp, 0);
    Ok(false)
}

fn rthteop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let tc = duel.get_first_target();
    duel.hint(HINT_SELECTMSG, tp, aux::stringid(ID, 3));
    let g = duel.select_matching_card(tp, |c, duel| rthtefilter(c, duel, None), tp, LOCATION_MZONE, 0, 1, 1, None)?;
    if g.is_empty() {
        return Ok(());
    }
    duel.hint_selection(g);
    if duel.send_to_hand(g, PLAYER_NONE, REASON_EFFECT)? > 0
        && g.get_first().is_some_and(|sc| sc.is_location(duel, LOCATION_HAND | LOCATION_EXTRA))
        && duel.get_location_count(tp, LOCATION_MZONE) > 0
        && let Some(tc) = tc
        && tc.is_relate_to_effect(duel, e)
        && duel.special_summon(tc, 0, tp, tp, false, false, POS_FACEUP)? > 0
    {
        tc.negate_effects(duel, e.get_handler(duel), 0, false);
    }
    Ok(())
}

fn thfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_ARTMAGE]) && c.is_spell(duel) && c.is_faceup(duel) && c.is_able_to_hand(duel)
}

fn thtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    stock::exists_target(duel, a, chk, thfilter, LOCATION_GRAVE | LOCATION_REMOVED, CATEGORY_TOHAND)
}

fn thop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_ATOHAND);
    let g = duel.select_matching_card(
        tp,
        aux::necro_valley_filter(thfilter),
        tp,
        LOCATION_GRAVE | LOCATION_REMOVED,
        0,
        1,
        1,
        None,
    )?;
    if !g.is_empty() {
        duel.hint_selection(g);
        duel.send_to_hand(g, PLAYER_NONE, REASON_EFFECT)?;
    }
    Ok(())
}
