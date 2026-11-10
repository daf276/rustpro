// Longirsu, the Orcust Orchestrator
use super::c30741503::Procedure;
use crate::prelude::*;
use crate::testing::{Check, EXTRA, MZONE, OPPONENT, REMOVED};

const ID: u32 = 76145142;

// In the Extra Deck for its Link Summon from Galatea and Girsu on the field, Machines banished to shuffle, and linked opponent monsters.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (30741503, MZONE),
    (69811710, MZONE),
    (69811710, MZONE),
    (69811710, REMOVED),
    (69811710, REMOVED),
    (30741503, MZONE | OPPONENT),
    (30741503, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[Check::SpecialSummoned, Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Link::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_property(duel, EFFECT_FLAG_SINGLE_RANGE);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_code(duel, EFFECT_INDESTRUCTABLE_EFFECT);
    e1.set_condition(duel, |duel, a| a.e.get_handler(duel).is_linked(duel));
    e1.set_value(duel, 1);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 0));
    e2.set_category(duel, CATEGORY_TODECK + CATEGORY_TOGRAVE);
    e2.set_type(duel, EFFECT_TYPE_IGNITION);
    e2.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_count_limit(duel, 1, ID);
    e2.set_condition(duel, |duel, a| !babelcon(duel, a));
    e2.set_cost(duel, tdcost);
    e2.set_target(duel, tdtg);
    e2.set_operation(duel, tdop);
    c.register_effect(duel, e2);
    let e3 = e2.clone_effect(duel);
    e3.set_type(duel, EFFECT_TYPE_QUICK_O);
    e3.set_code(duel, EVENT_FREE_CHAIN);
    e3.set_hint_timing(duel, 0, TIMINGS_CHECK_MONSTER);
    e3.set_condition(duel, babelcon);
    c.register_effect(duel, e3);
}

fn babelcon(duel: &Duel, a: &Args) -> bool {
    duel.is_player_affected_by_effect(a.tp, CARD_ORCUSTRATED_BABEL).is_some()
}

fn tdcost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let c = a.e.get_handler(duel);
    if chk == 0 {
        return Ok(c.get_attack_announced_count(duel) == 0);
    }
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE + EFFECT_FLAG_OATH);
    e1.set_code(duel, EFFECT_CANNOT_ATTACK);
    e1.set_reset(duel, RESETS_STANDARD_PHASE_END);
    c.register_effect(duel, e1);
    Ok(false)
}

fn tdfilter(c: CardId, duel: &Duel) -> bool {
    c.is_faceup(duel) && c.is_race(duel, RACE_MACHINE) && c.is_able_to_deck(duel)
}

fn tdtg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_controler(duel, tp) && chkc.is_location(duel, LOCATION_REMOVED) && tdfilter(chkc, duel));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(tdfilter, tp, LOCATION_REMOVED, 0, 2, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TODECK);
    let g = duel.select_target(tp, tdfilter, tp, LOCATION_REMOVED, 0, 2, 2, None)?;
    let count = g.map_or(0, |g| g.len() as u8);
    duel.set_operation_info(0, CATEGORY_TODECK, g, count, 0, 0);
    Ok(false)
}

fn tdop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    let tg = duel.get_target_cards(a.e).unwrap_or_default();
    if tg.is_empty() {
        return Ok(());
    }
    duel.send_to_deck(tg, PLAYER_NONE, SEQ_DECKTOP, REASON_EFFECT)?;
    let og = duel.get_operated_group();
    if og.is_empty() {
        return Ok(());
    }
    if og.is_exists(duel, |c, duel| c.is_location(duel, LOCATION_DECK), 1, None) {
        duel.shuffle_deck(tp)?;
    }
    let g = duel.get_matching_group(CardId::is_linked, tp, 0, LOCATION_MZONE, None);
    if !g.is_empty() && duel.select_yes_no(tp, aux::stringid(ID, 1))? {
        duel.break_effect()?;
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_TOGRAVE);
        let sg = g.select(duel, tp, 1, 1, None)?;
        duel.send_to_grave(sg, REASON_EFFECT)?;
    }
    Ok(())
}
