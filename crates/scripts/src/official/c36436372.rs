// Gimmick Puppet Little Soldiers
use crate::prelude::*;
use crate::testing::{Check, DECK, GRAVE, HAND, MZONE};

const ID: u32 = 36436372;

pub const LISTED_SERIES: &[u16] = &[SET_GIMMICK_PUPPET];

// In the hand to Normal Summon, Dreary Doll (Level 8) in the Deck to send, and Scissor Arms on the field to raise.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, HAND),
    (ID, GRAVE),
    (92418590, DECK),
    (92418590, DECK),
    (92418590, DECK),
    (29216967, MZONE),
    (29216967, MZONE),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_LVCHANGE);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e1.set_property(duel, EFFECT_FLAG_DELAY);
    e1.set_code(duel, EVENT_SUMMON_SUCCESS);
    e1.set_count_limit(duel, 1, ID);
    e1.set_cost(duel, selflvcost);
    e1.set_target(duel, selflvtg);
    e1.set_operation(duel, selflvop);
    c.register_effect(duel, e1);
    let e2 = e1.clone_effect(duel);
    e2.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 1));
    e3.set_category(duel, CATEGORY_LVCHANGE);
    e3.set_type(duel, EFFECT_TYPE_IGNITION);
    e3.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e3.set_range(duel, LOCATION_GRAVE);
    e3.set_cost(duel, Cost::self_banish);
    e3.set_target(duel, gplvtg);
    e3.set_operation(duel, gplvop);
    c.register_effect(duel, e3);
}

fn lvcostfilter(c: CardId, duel: &Duel, lv: u32) -> bool {
    c.is_set_card(duel, &[SET_GIMMICK_PUPPET]) && c.is_monster(duel) && !c.is_level(duel, &[lv]) && c.is_able_to_grave_as_cost(duel)
}

fn selflvcost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let tp = a.tp;
    let lv = a.e.get_handler(duel).get_level(duel);
    let filter = |c, duel: &Duel| lvcostfilter(c, duel, lv);
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(filter, tp, LOCATION_DECK, 0, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TOGRAVE);
    let g = duel.select_matching_card(tp, filter, tp, LOCATION_DECK, 0, 1, 1, None)?;
    duel.send_to_grave(g, REASON_COST)?;
    let level = g.get_first().map_or(0, |tc| tc.get_level(duel));
    a.e.set_label(duel, i64::from(level));
    Ok(false)
}

fn selflvtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(true);
    }
    let c = a.e.get_handler(duel);
    let level = a.e.get_label(duel) as i32;
    duel.set_operation_info(0, CATEGORY_LVCHANGE, Some(c.into()), 1, a.tp, level);
    Ok(false)
}

fn selflvop(duel: &mut Duel, a: &Args) -> Script<()> {
    let e = a.e;
    let c = e.get_handler(duel);
    if c.is_relate_to_effect(duel, e) && c.is_faceup(duel) {
        let level = e.get_label(duel) as i32;
        let e1 = duel.create_effect(c);
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_code(duel, EFFECT_CHANGE_LEVEL);
        e1.set_value(duel, level);
        e1.set_reset(duel, RESET_EVENT | RESETS_STANDARD_DISABLE);
        c.register_effect(duel, e1);
    }
    Ok(())
}

fn lvtgfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_GIMMICK_PUPPET]) && c.is_faceup(duel) && c.has_level(duel)
}

fn gplvtg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_controler(duel, tp) && chkc.is_location(duel, LOCATION_MZONE) && lvtgfilter(chkc, duel));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(lvtgfilter, tp, LOCATION_MZONE, 0, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TARGET);
    let g = duel.select_target(tp, lvtgfilter, tp, LOCATION_MZONE, 0, 1, 2, None)?;
    let count = g.map_or(0, |g| g.len() as u8);
    duel.set_operation_info(0, CATEGORY_LVCHANGE, g, count, tp, 4);
    Ok(false)
}

fn gplvop(duel: &mut Duel, a: &Args) -> Script<()> {
    let g = duel.get_target_cards(a.e).unwrap_or_default();
    if g.is_empty() {
        return Ok(());
    }
    let c = a.e.get_handler(duel);
    for tc in g {
        let e1 = duel.create_effect(c);
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE);
        e1.set_code(duel, EFFECT_UPDATE_LEVEL);
        e1.set_value(duel, 4);
        e1.set_reset(duel, RESETS_STANDARD_PHASE_END);
        tc.register_effect(duel, e1);
    }
    Ok(())
}
