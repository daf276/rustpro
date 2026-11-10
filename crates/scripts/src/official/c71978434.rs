// Deep Sea Minstrel
use crate::prelude::*;
use crate::testing::{Check, DECK, GRAVE, HAND};

const ID: u32 = 71978434;
const DEEP_SEA_DIVA: u32 = 78868119;

pub const LISTED_NAMES: &[u32] = &[ID];

// In the hand and the Deck, with Deep Sea Diva in the hand to Special Summon it, WATER monsters to discard and Atlantean
// Pikemen in the Graveyard to return.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, DECK),
    (ID, DECK),
    (DEEP_SEA_DIVA, HAND),
    (DEEP_SEA_DIVA, HAND),
    (26976414, GRAVE),
    (26976414, GRAVE),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_REMOVE);
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_property(duel, EFFECT_FLAG_PLAYER_TARGET);
    e1.set_range(duel, LOCATION_HAND);
    e1.set_count_limit(duel, 1, ID);
    e1.set_cost(duel, rmcost);
    e1.set_target(duel, rmtg);
    e1.set_operation(duel, rmop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_TODECK);
    e2.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e2.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e2.set_property(duel, EFFECT_FLAG_CARD_TARGET + EFFECT_FLAG_DELAY);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_cost(duel, tdcost);
    e2.set_target(duel, tdtg);
    e2.set_operation(duel, tdop);
    c.register_effect(duel, e2);
}

fn cfilter(c: CardId, duel: &Duel) -> bool {
    c.is_attribute(duel, ATTRIBUTE_WATER) && c.is_discardable(duel)
}

fn rmcost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let (c, tp) = (a.e.get_handler(duel), a.tp);
    if chk == 0 {
        return Ok(c.is_discardable(duel) && duel.is_existing_matching_card(cfilter, tp, LOCATION_HAND, 0, 1, Some(c.into())));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_DISCARD);
    let mut g = duel.select_matching_card(tp, cfilter, tp, LOCATION_HAND, 0, 1, 1, Some(c.into()))?;
    g.insert(c);
    duel.send_to_grave(g, REASON_DISCARD | REASON_COST)?;
    Ok(false)
}

fn rmtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(CardId::is_able_to_remove, tp, 0, LOCATION_HAND, 1, None));
    }
    duel.set_operation_info(0, CATEGORY_REMOVE, None, 1, 1 - tp, LOCATION_HAND as i32);
    Ok(false)
}

fn rmop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let hg = duel.get_field_group(tp, 0, LOCATION_HAND);
    if duel.is_player_affected_by_effect(1 - tp, 30459350).is_some() || hg.is_empty() {
        return Ok(());
    }
    duel.confirm_cards(tp, hg)?;
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_REMOVE);
    let g = hg.select(duel, tp, 1, 1, None)?;
    let Some(tc) = g.get_first() else { return Ok(()) };
    duel.remove(g, POS_FACEUP, REASON_EFFECT)?;
    duel.shuffle_hand(1 - tp)?;
    let c = e.get_handler(duel);
    let fid = i64::from(c.get_field_id(duel));
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_CONTINUOUS);
    e1.set_code(duel, EVENT_PHASE + PHASE_END);
    e1.set_count_limit(duel, 1, 0);
    e1.set_label(duel, fid);
    e1.set_label_object_group(duel, Some(tc.into()));
    e1.set_condition_mut(duel, retcon);
    e1.set_operation(duel, retop);
    e1.set_reset(duel, RESET_PHASE | PHASE_END);
    duel.register_effect(e1, tp);
    tc.register_flag_effect_full(duel, ID, RESETS_STANDARD_PHASE_END, 0, 1, fid, 0);
    Ok(())
}

fn retcon(duel: &mut Duel, a: &Args) -> bool {
    let tc = a.e.get_label_object_group(duel).and_then(|g| g.get_first());
    if tc.and_then(|tc| tc.get_flag_effect_label(duel, ID)) == Some(a.e.get_label(duel)) {
        return true;
    }
    a.e.reset(duel);
    false
}

fn retop(duel: &mut Duel, a: &Args) -> Script<()> {
    if let Some(tc) = a.e.get_label_object_group(duel).and_then(|g| g.get_first()) {
        duel.send_to_hand(tc, PLAYER_NONE, REASON_EFFECT)?;
    }
    Ok(())
}

fn tdcost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    if chk == 0 {
        return Ok(duel.is_player_can_discard_deck_as_cost(a.tp, 3));
    }
    duel.discard_deck(a.tp, 3, REASON_COST)?;
    Ok(false)
}

fn tdfilter(c: CardId, duel: &Duel) -> bool {
    c.is_level_below(duel, 4) && c.is_attribute(duel, ATTRIBUTE_WATER) && c.is_able_to_deck(duel) && !c.is_code(duel, ID)
}

fn tdtg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_GRAVE) && chkc.is_controler(duel, tp) && tdfilter(chkc, duel));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(tdfilter, tp, LOCATION_GRAVE, 0, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TODECK);
    let g = duel.select_target(tp, tdfilter, tp, LOCATION_GRAVE, 0, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_TODECK, g, 1, 0, 0);
    Ok(false)
}

fn tdop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tg: Option<CardSet> = duel.get_chain_info(0, CHAININFO_TARGET_CARDS);
    let sg = tg
        .unwrap_or_default()
        .filter(duel, |c, duel| c.is_relate_to_effect(duel, a.e), None);
    if !sg.is_empty() {
        let sequence = if duel.select_option(a.tp, &[aux::stringid(ID, 2), aux::stringid(ID, 3)])? == 0 {
            SEQ_DECKTOP
        } else {
            SEQ_DECKBOTTOM
        };
        duel.send_to_deck(sg, PLAYER_NONE, sequence, REASON_EFFECT)?;
    }
    Ok(())
}
