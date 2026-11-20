// Raise Moon Hope Squeeze - Jackpot
use crate::prelude::*;
use crate::testing::{Check, EXTRA, HAND, MZONE, OPPONENT};

const ID: u32 = 100458023;

struct Procedure;

impl XyzProcedure for Procedure {
    const LV: Option<u32> = Some(7);
    const CT: usize = 2;
    const MAXCT: usize = Xyz::INFINITE_MATS;
}

// On the field for the opponent's searches, in the Extra Deck with Level 7 materials, The Glitter of Raise Moon to Special
// Summon it with a material, Raise Moon Wings Rapi to Special Summon from the hand, and opponent's cards to place in the Deck.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (ID, EXTRA),
    (ID, EXTRA),
    (100458018, MZONE),
    (100458018, MZONE),
    (100458017, MZONE),
    (100458026, HAND),
    (100458018, HAND),
    (26202165, MZONE | OPPONENT),
    (26202165, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Xyz::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_code(duel, EFFECT_CANNOT_SELECT_BATTLE_TARGET);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_target_range(duel, 0, LOCATION_MZONE);
    e1.set_value(duel, Value::Card(|duel, e, c| i32::from(c != e.get_handler(duel))));
    c.register_effect(duel, e1);
    let e2a = duel.create_effect(c);
    e2a.set_description(duel, aux::stringid(ID, 0));
    e2a.set_category(duel, CATEGORY_TODECK);
    e2a.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e2a.set_property(duel, EFFECT_FLAG_DELAY);
    e2a.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e2a.set_count_limit(duel, 1, ID);
    e2a.set_condition(duel, |duel, a| a.e.get_handler(duel).is_xyz_summoned(duel));
    e2a.set_cost(duel, |duel, a, chk| {
        let max = duel.get_matching_group_count(|c, duel| c.is_able_to_deck(duel), a.tp, 0, LOCATION_ONFIELD, None);
        Cost::detach_from_self_with(duel, a, chk, 1, max as u16, |_, _, _| {})
    });
    e2a.set_target(duel, tdtg);
    e2a.set_operation(duel, tdop);
    c.register_effect(duel, e2a);
    let e2b = e2a.clone_effect(duel);
    e2b.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e2b.set_range(duel, LOCATION_MZONE);
    e2b.set_condition(duel, |duel, a| {
        a.eg.unwrap_or_default()
            .is_exists(duel, |c, duel| c.is_summon_location(duel, LOCATION_HAND), 1, None)
    });
    c.register_effect(duel, e2b);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 1));
    e3.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e3.set_property_full(duel, EFFECT_FLAG_DELAY, EFFECT_FLAG2_CHECK_SIMULTANEOUS);
    e3.set_code(duel, EVENT_MOVE);
    e3.set_range(duel, LOCATION_MZONE);
    e3.set_condition(duel, attachcon);
    e3.set_target(duel, attachtg);
    e3.set_operation(duel, attachop);
    c.register_effect(duel, e3);
}

fn detached_count(duel: &Duel, e: EffectId) -> usize {
    e.get_chain_data_group(duel, COST_DETACHED_MATERIALS).map_or(0, |g| g.len())
}

fn tdtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(|c, duel| c.is_able_to_deck(duel), tp, 0, LOCATION_ONFIELD, 1, None));
    }
    let count = detached_count(duel, a.e);
    duel.set_operation_info(0, CATEGORY_TODECK, None, count as u8, 1 - tp, LOCATION_ONFIELD as i32);
    Ok(false)
}

fn tdop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    let count = detached_count(duel, a.e);
    let filter = |c: CardId, duel: &Duel| c.is_able_to_deck(duel);
    if !duel.is_existing_matching_card(filter, tp, 0, LOCATION_ONFIELD, count, None) {
        return Ok(());
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TODECK);
    let mut g = duel.select_matching_card(tp, filter, tp, 0, LOCATION_ONFIELD, count as u8, count as u8, None)?;
    if g.is_empty() {
        return Ok(());
    }
    duel.hint_selection(g);
    if duel.send_to_deck(g, PLAYER_NONE, SEQ_DECKBOTTOM, REASON_EFFECT)? == 0 {
        return Ok(());
    }
    g.keep_matching(duel, |c, duel| c.is_location(duel, LOCATION_DECK), None);
    if g.is_empty() {
        return Ok(());
    }
    let (your_g, opp_g) = g.split(duel, |c, duel| c.is_controler(duel, tp), None);
    if your_g.len() > 1 {
        duel.sort_deckbottom(tp, tp, your_g.len() as u16)?;
    }
    if opp_g.len() > 1 {
        duel.sort_deckbottom(tp, 1 - tp, opp_g.len() as u16)?;
    }
    Ok(())
}

fn attachconfilter(c: CardId, duel: &Duel, opp: u8) -> bool {
    c.is_previous_controler(duel, opp)
        && c.is_previous_location(duel, LOCATION_DECK)
        && c.is_reason_player(duel, opp)
        && !c.is_reason(duel, REASON_DRAW)
}

fn attachcon(duel: &Duel, a: &Args) -> bool {
    let (c, tp) = (a.e.get_handler(duel), a.tp);
    let eg = a.eg.unwrap_or_default();
    eg.is_exists(duel, |ec, duel| attachconfilter(ec, duel, 1 - tp), 1, None)
        && (!eg.is_contains(c) || c.is_previous_location(duel, LOCATION_MZONE))
}

fn attachtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(a.e.get_handler(duel).is_xyz_monster(duel) && duel.get_field_group_count(1 - a.tp, LOCATION_DECK, 0) > 0);
    }
    Ok(false)
}

fn attachop(duel: &mut Duel, a: &Args) -> Script<()> {
    let c = a.e.get_handler(duel);
    let g = duel.get_decktop_group(1 - a.tp, 1);
    if c.is_relate_to_effect(duel, a.e) && !g.is_empty() {
        duel.disable_shuffle_check(true);
        duel.overlay(c, g)?;
    }
    Ok(())
}
