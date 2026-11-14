// Darklord Ixchel
use crate::prelude::*;
use crate::testing::{Check, HAND};

const ID: u32 = 52840267;

pub const LISTED_SERIES: &[u16] = &[SET_DARKLORD];
// In the hand with Darklord Desire to discard. No Darklord Spell/Trap is ported yet, so the second effect never has a target.
pub const SUPPORT: &[(u32, u16)] = &[(ID, HAND), (55690251, HAND), (55690251, HAND)];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::Report(&Check::ResolvesEffect(aux::stringid(ID, 1))),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.set_sp_summon_once(duel, ID);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_DRAW);
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_property(duel, EFFECT_FLAG_PLAYER_TARGET);
    e1.set_range(duel, LOCATION_HAND);
    e1.set_count_limit(duel, 1, ID);
    e1.set_cost(duel, drcost);
    e1.set_target(duel, stock::draw_target::<2>);
    e1.set_operation(duel, stock::draw);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_TODECK);
    e2.set_type(duel, EFFECT_TYPE_QUICK_O);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_code(duel, EVENT_FREE_CHAIN);
    e2.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_cost(duel, Cost::pay_lp::<1000>);
    e2.set_target(duel, cptg);
    e2.set_operation(duel, cpop);
    c.register_effect(duel, e2);
}

fn cfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_DARKLORD]) && c.is_discardable(duel)
}

fn drcost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let (tp, c) = (a.tp, a.e.get_handler(duel));
    if chk == 0 {
        return Ok(c.is_discardable(duel) && duel.is_existing_matching_card(cfilter, tp, LOCATION_HAND, 0, 1, Some(c.into())));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_DISCARD);
    let mut g = duel.select_matching_card(tp, cfilter, tp, LOCATION_HAND, 0, 1, 1, Some(c.into()))?;
    g.insert(c);
    duel.send_to_grave(g, REASON_DISCARD | REASON_COST)?;
    Ok(false)
}

fn cpfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_DARKLORD])
        && c.is_spell_trap(duel)
        && c.is_able_to_deck(duel)
        && c.check_activate_effect(duel, false, true).is_some()
}

fn cptg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_GRAVE) && chkc.is_controler(duel, tp) && cpfilter(chkc, duel));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(cpfilter, tp, LOCATION_GRAVE, 0, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TARGET);
    let g = duel.select_target(tp, cpfilter, tp, LOCATION_GRAVE, 0, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_TODECK, g, 1, 0, 0);
    Ok(false)
}

fn cpop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tc = duel.get_first_target();
    if !tc.is_some_and(|tc| tc.is_relate_to_effect(duel, a.e)) {
        return Ok(());
    }
    // Copying the Spell/Trap's effect (CheckActivateEffect with copy_info) is left out until a Darklord Spell/Trap is ported.
    unported!(duel, true);
    Ok(())
}
