// Kyoro Ryu-Ge Kaiva
use super::c55154344::to_deck_cost;
use super::c56499179::thgatefilter;
use crate::prelude::*;
use crate::testing::{Check, DECK, HAND, MZONE, OPPONENT};

const ID: u32 = 93509766;
const DINO_DOMAINS: u32 = 82661630;

pub const LISTED_NAMES: &[u32] = &[DINO_DOMAINS];
// Ryu-Ge Realm - Dino Domains in the Deck to search and in the hand to activate and return as the cost, copies in the hand
// and on the field, and the opponent's Raigeki and Dark Hole to destroy two cards.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, HAND),
    (ID, MZONE),
    (DINO_DOMAINS, DECK),
    (DINO_DOMAINS, DECK),
    (DINO_DOMAINS, HAND),
    (12580477, HAND | OPPONENT),
    (53129443, HAND | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
    Check::ResolvesEffect(aux::stringid(ID, 2)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_SEARCH + CATEGORY_TOHAND);
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_range(duel, LOCATION_HAND);
    e1.set_count_limit(duel, 1, ID);
    e1.set_cost(duel, Cost::self_to_deck);
    e1.set_target(duel, thtg);
    e1.set_operation(duel, thop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e2.set_type(duel, EFFECT_TYPE_QUICK_O);
    e2.set_code(duel, EVENT_FREE_CHAIN);
    e2.set_range(duel, LOCATION_HAND);
    e2.set_hint_timing(duel, 0, TIMING_MAIN_END | TIMINGS_CHECK_MONSTER);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_condition(duel, |duel, _| duel.is_main_phase() && duel.has_flag_effect_full(0, ID, 2));
    e2.set_target(duel, stock::special_summon_self_target);
    e2.set_operation(duel, stock::special_summon_self::<POS_FACEUP>);
    c.register_effect(duel, e2);
    aux::global_check(duel, ID, |duel| {
        let ge1 = duel.create_effect(c);
        ge1.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_CONTINUOUS);
        ge1.set_code(duel, EVENT_DESTROYED);
        ge1.set_operation(duel, checkop);
        duel.register_effect(ge1, 0);
    });
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 2));
    e3.set_category(duel, CATEGORY_DESTROY);
    e3.set_type(duel, EFFECT_TYPE_IGNITION);
    e3.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e3.set_range(duel, LOCATION_MZONE);
    e3.set_count_limit(duel, 1, (ID, 2));
    e3.set_cost(duel, descost);
    e3.set_target(duel, destg);
    e3.set_operation(duel, desop);
    c.register_effect(duel, e3);
}

fn checkop(duel: &mut Duel, a: &Args) -> Script<()> {
    for _ in 0..a.eg.unwrap_or_default().len() {
        duel.register_flag_effect(0, ID, RESET_PHASE | PHASE_END, 0, 1);
    }
    Ok(())
}

fn thtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    stock::exists_target(duel, a, chk, thgatefilter::<DINO_DOMAINS>, LOCATION_DECK, CATEGORY_TOHAND)
}

fn thop(duel: &mut Duel, a: &Args) -> Script<()> {
    stock::search(duel, a, thgatefilter::<DINO_DOMAINS>, LOCATION_DECK)
}

fn tdfilter(c: CardId, duel: &Duel, ec: CardId) -> bool {
    c.is_code(duel, DINO_DOMAINS)
        && c.is_faceup(duel)
        && c.is_able_to_deck_as_cost(duel)
        && duel.is_existing_target(
            aux::TRUE,
            0,
            LOCATION_ONFIELD,
            LOCATION_ONFIELD,
            1,
            Some(CardSet::from_iter([c, ec])),
        )
}

fn descost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let c = a.e.get_handler(duel);
    to_deck_cost(duel, a, chk, |tc, duel| tdfilter(tc, duel, c))
}

fn destg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    let c = a.e.get_handler(duel);
    if let Some(chkc) = chkc {
        return Ok(chkc.is_on_field(duel) && chkc != c);
    }
    let ct = duel
        .get_matching_group(CardId::is_faceup, tp, LOCATION_MZONE, LOCATION_MZONE, None)
        .get_class_count(duel, |c, duel| c.get_race(duel) as i64);
    if chk == 0 {
        return Ok(ct > 0 && duel.is_existing_target(aux::TRUE, tp, LOCATION_ONFIELD, LOCATION_ONFIELD, 1, Some(c.into())));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_DESTROY);
    let g = duel.select_target(tp, aux::TRUE, tp, LOCATION_ONFIELD, LOCATION_ONFIELD, 1, ct as u8, Some(c.into()))?;
    let count = g.map_or(0, |g| g.len());
    duel.set_operation_info(0, CATEGORY_DESTROY, g, count as u8, tp, 0);
    Ok(false)
}

fn desop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tg = duel.get_target_cards(a.e).unwrap_or_default();
    if !tg.is_empty() {
        duel.destroy(tg, REASON_EFFECT)?;
    }
    Ok(())
}
