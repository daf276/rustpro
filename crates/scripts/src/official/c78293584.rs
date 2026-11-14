// Filia Diabell
use crate::prelude::*;
use crate::testing::{Check, DECK, GRAVE, HAND, MZONE};

const ID: u32 = 78293584;

pub const LISTED_SERIES: &[u16] = &[SET_DIABELL];
// In the hand and GY, with Vengeance in the hand to banish it, Diabellze and Vengeance in the Deck to search and Black Witch on the field.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, GRAVE),
    (64998567, GRAVE),
    (23151193, HAND),
    (60145298, DECK),
    (60145298, DECK),
    (23151193, DECK),
    (72270339, MZONE),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_TOHAND + CATEGORY_SEARCH + CATEGORY_SPECIAL_SUMMON);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_count_limit(duel, 1, ID);
    e1.set_target(duel, thsptg);
    e1.set_operation(duel, thspop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_ATKCHANGE);
    e2.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e2.set_property(duel, EFFECT_FLAG_DELAY);
    e2.set_code(duel, EVENT_REMOVE);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_target(duel, atktg);
    e2.set_operation(duel, atkop);
    c.register_effect(duel, e2);
}

fn thspfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8, sp_check: bool) -> bool {
    c.is_set_card(duel, &[SET_DIABELL])
        && c.is_level_above(duel, 8)
        && (c.is_able_to_hand(duel) || (sp_check && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)))
}

fn only_opponent_controls(duel: &Duel, tp: u8) -> bool {
    duel.get_location_count(tp, LOCATION_MZONE) > 0
        && duel.get_field_group_count(tp, LOCATION_MZONE, 0) == 0
        && duel.get_field_group_count(tp, 0, LOCATION_MZONE) > 0
}

fn thsptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if chk == 0 {
        let sp_check = only_opponent_controls(duel, tp);
        return Ok(duel.is_existing_matching_card(|c, duel| thspfilter(c, duel, e, tp, sp_check), tp, LOCATION_DECK, 0, 1, None));
    }
    duel.set_possible_operation_info(0, CATEGORY_TOHAND, None, 1, tp, LOCATION_DECK as i32);
    duel.set_possible_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, tp, LOCATION_DECK as i32);
    Ok(false)
}

fn thspop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let sp_check = only_opponent_controls(duel, tp);
    duel.hint(HINT_SELECTMSG, tp, if sp_check { aux::stringid(ID, 2) } else { HINTMSG_ATOHAND });
    let g = duel.select_matching_card(tp, |c, duel| thspfilter(c, duel, e, tp, sp_check), tp, LOCATION_DECK, 0, 1, 1, None)?;
    let Some(sc) = g.get_first() else { return Ok(()) };
    if sp_check {
        aux::to_hand_or_else(
            duel,
            sc,
            tp,
            |sc, duel| sp_check && sc.is_can_be_special_summoned(duel, e, 0, tp, false, false),
            |duel| duel.special_summon(sc, 0, tp, tp, false, false, POS_FACEUP).map(drop),
            aux::stringid(ID, 3),
        )?;
    } else {
        duel.send_to_hand(sc, PLAYER_NONE, REASON_EFFECT)?;
        duel.confirm_cards(1 - tp, sc)?;
    }
    Ok(())
}

fn diabellmonsters(duel: &Duel, tp: u8) -> CardSet {
    duel.get_matching_group(
        aux::faceup_filter(|c, duel| c.is_set_card(duel, &[SET_DIABELL])),
        tp,
        LOCATION_MZONE,
        0,
        None,
    )
}

fn atktg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    let g = diabellmonsters(duel, tp);
    if chk == 0 {
        return Ok(!g.is_empty());
    }
    duel.set_possible_operation_info(0, CATEGORY_ATKCHANGE, Some(g), g.len() as u8, tp, 500);
    Ok(false)
}

fn atkop(duel: &mut Duel, a: &Args) -> Script<()> {
    let g = diabellmonsters(duel, a.tp);
    let c = a.e.get_handler(duel);
    for tc in g {
        let e1 = duel.create_effect(c);
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE);
        e1.set_code(duel, EFFECT_UPDATE_ATTACK);
        e1.set_value(duel, 500);
        e1.set_reset(duel, RESET_EVENT | RESETS_STANDARD);
        tc.register_effect(duel, e1);
    }
    Ok(())
}
