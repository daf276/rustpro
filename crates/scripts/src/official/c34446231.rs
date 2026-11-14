// Infernoid Flood
use crate::prelude::*;
use crate::testing::{Check, DECK, EXTRA, MZONE};

const ID: u32 = 34446231;

pub const LISTED_SERIES: &[u16] = &[SET_INFERNOID];

struct Procedure;

impl LinkProcedure for Procedure {
    const MIN: usize = 2;

    fn specialchk(duel: &Duel, sg: CardSet, lc: CardId, sumtype: u64, tp: u8) -> bool {
        sg.is_exists(
            duel,
            |c, duel| c.is_set_card_full(duel, &[SET_INFERNOID], Some(lc), sumtype, tp),
            1,
            None,
        )
    }
}

// Infernoid Decatron and Pirmais (Level 1) to Link Summon with, and Pirmais in the Deck to Special Summon.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (ID, EXTRA),
    (58446973, MZONE),
    (58446973, MZONE),
    (60551528, MZONE),
    (60551528, MZONE),
    (60551528, DECK),
    (60551528, DECK),
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
    e1.set_category(duel, CATEGORY_DISABLE_SUMMON + CATEGORY_REMOVE);
    e1.set_type(duel, EFFECT_TYPE_QUICK_O);
    e1.set_code(duel, EVENT_SPSUMMON);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_count_limit(duel, 1, ID);
    e1.set_condition(duel, |duel, a| a.ep == 1 - a.tp && duel.get_current_chain() == 0);
    e1.set_cost(duel, negsumcost);
    e1.set_target(duel, negsumtg);
    e1.set_operation(duel, negsumop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_REMOVE);
    e2.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e2.set_property(duel, EFFECT_FLAG_DELAY);
    e2.set_code(duel, EVENT_REMOVE);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_condition(duel, rmcon);
    e2.set_target(duel, rmtg);
    e2.set_operation(duel, rmop);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 2));
    e3.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e3.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e3.set_property(duel, EFFECT_FLAG_DELAY);
    e3.set_code(duel, EVENT_DESTROYED);
    e3.set_count_limit(duel, 1, (ID, 2));
    e3.set_condition(duel, spcon);
    e3.set_target(duel, sptg);
    e3.set_operation(duel, spop);
    c.register_effect(duel, e3);
}

fn negsumcost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.check_release_group_cost(tp, aux::TRUE, (1, 1), false, None, None));
    }
    let g = duel.select_release_group_cost(tp, aux::TRUE, (1, 1), false, None, None)?;
    duel.release(g, REASON_COST)?;
    Ok(false)
}

fn negsumtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    let eg = a.eg.unwrap_or_default();
    if chk == 0 {
        return Ok(eg.is_exists(duel, CardId::is_able_to_remove, 1, None));
    }
    duel.set_operation_info(0, CATEGORY_DISABLE_SUMMON, Some(eg), eg.len() as u8, tp, 0);
    duel.set_operation_info(0, CATEGORY_REMOVE, Some(eg), eg.len() as u8, tp, 0);
    Ok(false)
}

fn negsumop(duel: &mut Duel, a: &Args) -> Script<()> {
    let eg = a.eg.unwrap_or_default();
    duel.negate_summon(eg);
    duel.remove(eg, POS_FACEUP, REASON_EFFECT)?;
    Ok(())
}

fn rmcon(duel: &Duel, a: &Args) -> bool {
    let tp = a.tp;
    a.eg.unwrap_or_default().is_exists(
        duel,
        |c, duel| c.is_previous_location(duel, LOCATION_GRAVE) && c.is_previous_controler(duel, tp),
        1,
        None,
    )
}

fn rmtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let g = duel.get_matching_group(CardId::is_able_to_remove, a.tp, LOCATION_ONFIELD, LOCATION_ONFIELD, None);
    if chk == 0 {
        return Ok(!g.is_empty());
    }
    duel.set_operation_info(0, CATEGORY_REMOVE, Some(g), 1, PLAYER_EITHER, LOCATION_ONFIELD as i32);
    Ok(false)
}

fn rmop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_REMOVE);
    let g = duel.select_matching_card(tp, CardId::is_able_to_remove, tp, LOCATION_ONFIELD, LOCATION_ONFIELD, 1, 1, None)?;
    if !g.is_empty() {
        duel.hint_selection(g);
        duel.remove(g, POS_FACEUP, REASON_EFFECT)?;
    }
    Ok(())
}

fn spcon(duel: &Duel, a: &Args) -> bool {
    let (c, tp) = (a.e.get_handler(duel), a.tp);
    a.rp == 1 - tp && c.is_link_summoned(duel) && c.is_previous_controler(duel, tp) && c.is_previous_location(duel, LOCATION_MZONE)
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_set_card(duel, &[SET_INFERNOID]) && c.is_monster(duel) && c.is_can_be_special_summoned(duel, e, 0, tp, true, false)
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    stock::special_summon_target(duel, a, chk, |c, duel| spfilter(c, duel, a.e, a.tp), LOCATION_DECK)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
    let g = duel.select_matching_card(tp, |c, duel| spfilter(c, duel, e, tp), tp, LOCATION_DECK, 0, 1, 1, None)?;
    if !g.is_empty() {
        duel.special_summon(g, 0, tp, tp, true, false, POS_FACEUP)?;
    }
    Ok(())
}
