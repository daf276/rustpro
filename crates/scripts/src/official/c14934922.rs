// Salamangreat Rage
use crate::prelude::*;
use crate::testing::{Check, EXTRA, HAND, MZONE, SZONE};

const ID: u32 = 14934922;

pub const LISTED_SERIES: &[u16] = &[SET_SALAMANGREAT];

// Set, with Salamangreat Raging Phoenix on the field to send and in the Extra Deck, and Salamangreat Sanctuary to Link Summon it
// from a copy (a Reincarnation Summon).
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, SZONE),
    (ID, SZONE),
    (1295111, 0),
    (1295111, HAND),
    (57134592, MZONE),
    (57134592, MZONE),
    (57134592, EXTRA),
    (57134592, EXTRA),
    (57134592, EXTRA),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    aux::enable_check_reincarnation(duel);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_DESTROY);
    e1.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_count_limit_full(duel, 1, ID, EFFECT_COUNT_CODE_OATH);
    e1.set_cost(duel, cost);
    e1.set_target(duel, target1);
    e1.set_operation(duel, activate1);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_DESTROY);
    e2.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e2.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e2.set_code(duel, EVENT_FREE_CHAIN);
    e2.set_count_limit_full(duel, 1, ID, EFFECT_COUNT_CODE_OATH);
    e2.set_target(duel, target2);
    e2.set_operation(duel, activate2);
    c.register_effect(duel, e2);
}

fn costfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_SALAMANGREAT])
        && c.is_monster(duel)
        && (c.is_faceup(duel) || !c.is_location(duel, LOCATION_MZONE))
        && c.is_able_to_grave_as_cost(duel)
}

fn cost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    stock::send_to_grave_cost(duel, a, chk, costfilter, LOCATION_HAND | LOCATION_MZONE)
}

fn target1(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (tp, c) = (a.tp, a.e.get_handler(duel));
    if let Some(chkc) = chkc {
        return Ok(chkc.is_on_field(duel) && chkc != c);
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(aux::TRUE, tp, LOCATION_ONFIELD, LOCATION_ONFIELD, 1, Some(c.into())));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_DESTROY);
    let g = duel.select_target(tp, aux::TRUE, tp, LOCATION_ONFIELD, LOCATION_ONFIELD, 1, 1, Some(c.into()))?;
    duel.set_operation_info(0, CATEGORY_DESTROY, g, 1, 0, 0);
    Ok(false)
}

fn activate1(duel: &mut Duel, a: &Args) -> Script<()> {
    stock::destroy_first_target(duel, a)
}

fn lkfilter(c: CardId, duel: &Duel) -> bool {
    c.is_faceup(duel) && c.is_set_card(duel, &[SET_SALAMANGREAT]) && c.is_link_monster(duel) && c.is_reincarnation_summoned(duel)
}

fn target2(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_on_field(duel) && chkc != a.e.get_handler(duel));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(lkfilter, tp, LOCATION_MZONE, 0, 1, None)
            && duel.is_existing_matching_card(aux::TRUE, tp, 0, LOCATION_ONFIELD, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TARGET);
    let g = duel.select_target(tp, lkfilter, tp, LOCATION_MZONE, 0, 1, 1, None)?;
    let link = g.and_then(|g| g.get_first()).map_or(0, |tc| tc.get_link(duel));
    duel.set_operation_info(0, CATEGORY_DESTROY, g, link as u8, 0, 0);
    Ok(false)
}

fn activate2(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    let Some(tc) = duel.get_first_target() else { return Ok(()) };
    if !tc.is_relate_to_effect(duel, a.e) {
        return Ok(());
    }
    let ct = tc.get_link(duel);
    let g = duel.get_matching_group(aux::TRUE, tp, 0, LOCATION_ONFIELD, None);
    if !g.is_empty() {
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_DESTROY);
        let sg = g.select(duel, tp, 1, ct as u8, None)?;
        duel.hint_selection(sg);
        duel.destroy(sg, REASON_EFFECT)?;
    }
    Ok(())
}
