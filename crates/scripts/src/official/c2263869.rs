// Ultimate Slayer
use crate::prelude::*;
use crate::testing::{EXTRA, HAND, MZONE, OPPONENT};

const ID: u32 = 2263869;

// Extra Deck monsters to send as the cost, and the opponent's Extra Deck monsters of the same kind to return.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (1621413, EXTRA),
    (16195942, EXTRA),
    (1621413, MZONE | OPPONENT),
    (16195942, MZONE | OPPONENT),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_TODECK);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_cost(duel, tdcost);
    e1.set_target(duel, tdtg);
    e1.set_operation(duel, tdop);
    c.register_effect(duel, e1);
}

fn tdfilter(c: CardId, duel: &Duel, e: EffectId, extype: u32) -> bool {
    c.is_faceup(duel) && c.is_type(duel, extype) && c.is_able_to_deck(duel) && c.is_can_be_effect_target(duel, e)
}

fn tdcostfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    let extype = c.get_type(duel) & (TYPE_EXTRA | TYPE_PENDULUM);
    c.is_able_to_grave_as_cost(duel)
        && duel.is_existing_matching_card(|tc, duel| tdfilter(tc, duel, e, extype), tp, 0, LOCATION_MZONE, 1, None)
}

fn tdcost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(|c, duel| tdcostfilter(c, duel, e, tp), tp, LOCATION_EXTRA, 0, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TOGRAVE);
    let g = duel.select_matching_card(tp, |c, duel| tdcostfilter(c, duel, e, tp), tp, LOCATION_EXTRA, 0, 1, 1, None)?;
    duel.send_to_grave(g, REASON_COST)?;
    let extype = g.get_first().map_or(0, |tc| tc.get_type(duel) & (TYPE_EXTRA | TYPE_PENDULUM));
    e.set_label(duel, extype.into());
    Ok(false)
}

fn tdtg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let extype = e.get_label(duel) as u32;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_MZONE) && chkc.is_controler(duel, 1 - tp) && tdfilter(chkc, duel, e, extype));
    }
    if chk == 0 {
        return Ok(true);
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TODECK);
    let g = duel.select_target(tp, |c, duel| tdfilter(c, duel, e, extype), tp, 0, LOCATION_MZONE, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_TODECK, g, 1, 0, 0);
    if e.is_has_type(duel, EFFECT_TYPE_ACTIVATE) {
        duel.set_chain_limit(|duel, e, ep, tp, _| ep == tp || !e.is_monster_effect(duel), 0);
    }
    Ok(false)
}

fn tdop(duel: &mut Duel, a: &Args) -> Script<()> {
    if let Some(tc) = duel.get_first_target()
        && tc.is_relate_to_effect(duel, a.e)
    {
        duel.send_to_deck(tc, PLAYER_NONE, SEQ_DECKSHUFFLE, REASON_EFFECT)?;
    }
    Ok(())
}
