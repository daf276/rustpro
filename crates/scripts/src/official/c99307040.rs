// Horus the Black Flame Deity
use crate::prelude::*;
use crate::testing::{Check, HAND, MZONE, OPPONENT, SZONE};

const ID: u32 = 99307040;

pub const LISTED_SERIES: &[u16] = &[SET_HORUS];
pub const LISTED_NAMES: &[u32] = &[ID, CARD_KING_SARCOPHAGUS];

// In the hand with Imsety and King's Sarcophagus on the field, which the opponent's Raigeki and Dark Hole destroy.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, HAND),
    (84941194, MZONE),
    (84941194, MZONE),
    (CARD_KING_SARCOPHAGUS, SZONE),
    (12580477, HAND | OPPONENT),
    (53129443, HAND | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_TOGRAVE);
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_range(duel, LOCATION_HAND);
    e1.set_count_limit(duel, 1, ID);
    e1.set_condition(duel, tgcon);
    e1.set_cost(duel, tgcost);
    e1.set_target(duel, tgtg);
    e1.set_operation(duel, tgop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_SPECIAL_SUMMON + CATEGORY_TOGRAVE);
    e2.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e2.set_property(duel, EFFECT_FLAG_DELAY);
    e2.set_code(duel, EVENT_LEAVE_FIELD);
    e2.set_range(duel, LOCATION_HAND);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_condition(duel, spcon);
    e2.set_target(duel, sptg);
    e2.set_operation(duel, spop);
    c.register_effect(duel, e2);
}

fn tgcon(duel: &Duel, a: &Args) -> bool {
    let horus = aux::faceup_filter(|c, duel| c.is_set_card(duel, &[SET_HORUS]));
    let sarcophagus = aux::faceup_filter(|c, duel| c.is_code(duel, CARD_KING_SARCOPHAGUS));
    duel.is_existing_matching_card(horus, a.tp, LOCATION_MZONE, 0, 1, None)
        && duel.is_existing_matching_card(sarcophagus, a.tp, LOCATION_ONFIELD, 0, 1, None)
}

fn tgcost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let tp = a.tp;
    let c = a.e.get_handler(duel);
    let tg = duel.get_matching_group(CardId::is_able_to_grave, tp, LOCATION_ONFIELD, LOCATION_ONFIELD, None);
    let exc = (tg.len() == 1).then_some(tg);
    if chk == 0 {
        return Ok(!tg.is_empty()
            && !c.is_public(duel)
            && duel.is_existing_matching_card(CardId::is_able_to_grave_as_cost, tp, LOCATION_HAND | LOCATION_ONFIELD, 0, 1, exc));
    }
    duel.shuffle_hand(tp)?;
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TOGRAVE);
    let g = duel.select_matching_card(
        tp,
        CardId::is_able_to_grave_as_cost,
        tp,
        LOCATION_HAND | LOCATION_ONFIELD,
        0,
        1,
        1,
        exc,
    )?;
    duel.send_to_grave(g, REASON_COST)?;
    Ok(false)
}

fn tgtg(duel: &mut Duel, _: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(true);
    }
    duel.set_operation_info(0, CATEGORY_TOGRAVE, None, 1, PLAYER_EITHER, LOCATION_ONFIELD as i32);
    Ok(false)
}

fn tgop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TOGRAVE);
    let g = duel.select_matching_card(tp, CardId::is_able_to_grave, tp, LOCATION_ONFIELD, LOCATION_ONFIELD, 1, 1, None)?;
    if !g.is_empty() {
        duel.hint_selection_full(g, true);
        duel.send_to_grave(g, REASON_EFFECT)?;
    }
    Ok(())
}

fn spconfilter(c: CardId, duel: &Duel, tp: u8) -> bool {
    let (code1, code2) = (c.get_previous_code_on_field(duel), c.get_previous_code2_on_field(duel));
    c.is_previous_controler(duel, tp)
        && c.is_previous_position(duel, POS_FACEUP)
        && c.get_reason_player(duel) == 1 - tp
        && c.is_reason(duel, REASON_EFFECT)
        && (code1 == CARD_KING_SARCOPHAGUS
            || code2 == CARD_KING_SARCOPHAGUS
            || (c.is_previous_location(duel, LOCATION_MZONE) && c.is_previous_set_card(duel, &[SET_HORUS]) && code1 != ID && code2 != ID))
}

fn spcon(duel: &Duel, a: &Args) -> bool {
    let tp = a.tp;
    a.eg.unwrap_or_default()
        .is_exists(duel, |c, duel| spconfilter(c, duel, tp), 1, None)
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if chk == 0 {
        return Ok(duel.get_location_count(tp, LOCATION_MZONE) > 0 && c.is_can_be_special_summoned(duel, e, 0, tp, false, false));
    }
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, Some(c.into()), 1, 0, 0);
    duel.set_possible_operation_info(0, CATEGORY_TOGRAVE, None, 1, PLAYER_ALL, LOCATION_MZONE as i32);
    Ok(false)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if !c.is_relate_to_effect(duel, e) || duel.special_summon(c, 0, tp, tp, false, false, POS_FACEUP)? == 0 {
        return Ok(());
    }
    let g = duel.get_matching_group(CardId::is_able_to_grave, tp, LOCATION_MZONE, LOCATION_MZONE, Some(c.into()));
    if !g.is_empty() && duel.select_yes_no(tp, aux::stringid(ID, 2))? {
        duel.send_to_grave(g, REASON_EFFECT)?;
    }
    Ok(())
}
