// Ryu-Ge Rising
use crate::prelude::*;
use crate::testing::{Check, DECK, GRAVE, HAND, PZONE};

const ID: u32 = 29882827;

pub const LISTED_SERIES: &[u16] = &[SET_RYU_GE];
// In the hand to activate and in the Graveyard, with Ryu-Ge monsters in the Deck and Graveyard and Tensei Ryu-Ge Anva in a
// Pendulum Zone and in the Deck.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, GRAVE),
    (56499179, DECK),
    (56499179, DECK),
    (56322832, DECK),
    (56322832, DECK),
    (56499179, GRAVE),
    (56322832, GRAVE),
    (56322832, PZONE),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_TOHAND + CATEGORY_SEARCH + CATEGORY_TOEXTRA);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_hint_timing(duel, 0, TIMING_MAIN_END | TIMINGS_CHECK_MONSTER_E);
    e1.set_count_limit(duel, 1, ID);
    e1.set_target(duel, thtg);
    e1.set_operation(duel, thop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e2.set_type(duel, EFFECT_TYPE_IGNITION);
    e2.set_range(duel, LOCATION_GRAVE);
    e2.set_count_limit(duel, 1, ID);
    e2.set_condition(duel, |duel, a| {
        duel.is_existing_matching_card(spconfilter, a.tp, LOCATION_ONFIELD, 0, 1, None)
    });
    e2.set_cost(duel, spcost);
    e2.set_target(duel, sptg);
    e2.set_operation(duel, spop);
    c.register_effect(duel, e2);
}

fn thfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_RYU_GE]) && c.is_monster(duel) && c.is_able_to_hand(duel)
}

fn thtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(thfilter, tp, LOCATION_DECK, 0, 1, None));
    }
    duel.set_operation_info(0, CATEGORY_TOHAND, None, 1, tp, LOCATION_DECK as i32);
    duel.set_possible_operation_info(0, CATEGORY_TOEXTRA, None, 1, tp, LOCATION_HAND as i32);
    Ok(false)
}

fn tedfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_RYU_GE]) && c.is_type(duel, TYPE_PENDULUM)
}

fn thop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_ATOHAND);
    let Some(tc) = duel
        .select_matching_card(tp, thfilter, tp, LOCATION_DECK, 0, 1, 1, None)?
        .get_first()
    else {
        return Ok(());
    };
    if duel.send_to_hand(tc, PLAYER_NONE, REASON_EFFECT)? > 0 && tc.is_location(duel, LOCATION_HAND) {
        duel.confirm_cards(1 - tp, tc)?;
        duel.shuffle_hand(tp)?;
        if duel.is_existing_matching_card(tedfilter, tp, LOCATION_HAND, 0, 1, None) && duel.select_yes_no(tp, aux::stringid(ID, 2))? {
            duel.hint(HINT_SELECTMSG, tp, aux::stringid(ID, 3));
            let g = duel.select_matching_card(tp, tedfilter, tp, LOCATION_HAND, 0, 1, 1, None)?;
            if !g.is_empty() {
                duel.break_effect()?;
                duel.send_to_extra_p(g, tp, REASON_EFFECT)?;
            }
        }
    }
    Ok(())
}

fn spconfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_RYU_GE]) && c.is_original_type(duel, TYPE_PENDULUM) && c.is_faceup(duel)
}

fn spcostfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_set_card(duel, &[SET_RYU_GE])
        && c.is_monster(duel)
        && c.is_able_to_remove_as_cost(duel)
        && duel.is_existing_matching_card(
            |sc, duel| spfilter(sc, duel, e, tp, c.get_original_race(duel)),
            tp,
            LOCATION_DECK,
            0,
            1,
            None,
        )
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8, race: u64) -> bool {
    c.is_set_card(duel, &[SET_RYU_GE]) && c.is_original_race(duel, race) && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
}

fn spcost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if chk == 0 {
        return Ok(c.is_able_to_remove_as_cost(duel)
            && duel.is_existing_matching_card(|tc, duel| spcostfilter(tc, duel, e, tp), tp, LOCATION_GRAVE, 0, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_REMOVE);
    let mut g = duel.select_matching_card(tp, |tc, duel| spcostfilter(tc, duel, e, tp), tp, LOCATION_GRAVE, 0, 1, 1, None)?;
    let race = g.get_first().map_or(0, |tc| tc.get_original_race(duel));
    e.set_label(duel, race as i64);
    g.insert(c);
    duel.remove(g, POS_FACEUP, REASON_COST)?;
    Ok(false)
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.get_location_count(tp, LOCATION_MZONE) > 0);
    }
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, tp, LOCATION_DECK as i32);
    Ok(false)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let race = e.get_label(duel) as u64;
    stock::special_summon(duel, a, |c, duel| spfilter(c, duel, e, tp, race), LOCATION_DECK, POS_FACEUP)
}
