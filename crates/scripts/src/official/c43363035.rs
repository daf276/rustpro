// Wandering Titan of Tartarus
use crate::prelude::*;
use crate::testing::{Check, DECK, HAND, MZONE, OPPONENT};

const ID: u32 = 43363035;

// Copies in the Deck, milled by Charge of the Light Brigade (with Lyla to search), and in the hand to be discarded or destroyed by the
// opponent's Dark Hole.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, DECK),
    (ID, DECK),
    (ID, DECK),
    (ID, DECK),
    (ID, HAND),
    (94886282, HAND),
    (94886282, HAND),
    (22624373, DECK),
    (22624373, DECK),
    (6103114, MZONE),
    (26202165, MZONE | OPPONENT),
    (53129443, HAND | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_TOGRAVE + CATEGORY_SPECIAL_SUMMON);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_F);
    e1.set_code(duel, EVENT_TO_GRAVE);
    e1.set_condition(duel, |duel, a| a.e.get_handler(duel).is_previous_location(duel, LOCATION_DECK));
    e1.set_count_limit(duel, 1, ID);
    e1.set_target(duel, sptg);
    e1.set_operation(duel, spop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_TOGRAVE);
    e2.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e2.set_property(duel, EFFECT_FLAG_DELAY);
    e2.set_code(duel, EVENT_TO_GRAVE);
    e2.set_count_limit(duel, 1, ID);
    e2.set_condition(duel, |duel, a| !a.e.get_handler(duel).is_previous_location(duel, LOCATION_DECK));
    e2.set_cost(duel, tgcost);
    e2.set_target(duel, tgtg);
    e2.set_operation(duel, tgop);
    c.register_effect(duel, e2);
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(true);
    }
    let g = duel.get_field_group(a.tp, LOCATION_ONFIELD, LOCATION_ONFIELD);
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, Some(a.e.get_handler(duel).into()), 1, a.tp, 0);
    duel.set_operation_info(0, CATEGORY_TOGRAVE, Some(g), g.len() as u8, a.tp, 0);
    duel.set_chain_limit(|duel, e, _, _, _| !e.is_monster_effect(duel), 0);
    Ok(false)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    let your_fg = duel.get_matching_group(CardId::is_able_to_grave, tp, LOCATION_ONFIELD, 0, None);
    if your_fg.is_empty() || duel.send_to_grave(your_fg, REASON_EFFECT)? == 0 {
        return Ok(());
    }
    let c = a.e.get_handler(duel);
    let ct = your_fg.filter_count(duel, |c, duel| c.is_location(duel, LOCATION_GRAVE), None);
    if ct > 0 && c.is_relate_to_effect(duel, a.e) && duel.special_summon(c, 0, tp, tp, false, false, POS_FACEUP)? > 0 {
        let mut opp_fg = duel.get_matching_group(CardId::is_able_to_grave, tp, 0, LOCATION_ONFIELD, None);
        if opp_fg.is_empty() {
            return Ok(());
        }
        if opp_fg.len() > 1 {
            duel.hint(HINT_SELECTMSG, tp, HINTMSG_TOGRAVE);
            opp_fg = opp_fg.select(duel, tp, 1, ct.min(opp_fg.len()) as u8, None)?;
        }
        duel.hint_selection(opp_fg);
        duel.break_effect()?;
        duel.send_to_grave(opp_fg, REASON_EFFECT)?;
    }
    Ok(())
}

fn tgcost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(CardId::is_able_to_grave_as_cost, tp, LOCATION_HAND, 0, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TOGRAVE);
    let g = duel.select_matching_card(tp, CardId::is_able_to_grave_as_cost, tp, LOCATION_HAND, 0, 1, 1, None)?;
    duel.send_to_grave(g, REASON_COST)?;
    Ok(false)
}

fn tgtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(CardId::is_able_to_grave, tp, LOCATION_ONFIELD, LOCATION_ONFIELD, 1, None));
    }
    let g = duel.get_matching_group(CardId::is_able_to_grave, tp, LOCATION_ONFIELD, LOCATION_ONFIELD, None);
    duel.set_operation_info(0, CATEGORY_TOGRAVE, Some(g), 1, tp, 0);
    Ok(false)
}

fn tgop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TOGRAVE);
    let g = duel.select_matching_card(tp, CardId::is_able_to_grave, tp, LOCATION_ONFIELD, LOCATION_ONFIELD, 1, 1, None)?;
    if !g.is_empty() {
        duel.hint_selection(g);
        duel.send_to_grave(g, REASON_EFFECT)?;
    }
    Ok(())
}
