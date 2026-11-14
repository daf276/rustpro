// Interrupted Kaiju Slumber
use crate::prelude::*;
use crate::testing::{Check, DECK, GRAVE, HAND, MZONE, OPPONENT};

const ID: u32 = 99330325;

pub const LISTED_SERIES: &[u16] = &[SET_KAIJU];
// In the hand and Graveyard, with two Kaiju in the Deck to Special Summon (and one to add) and Sangan on both fields to destroy.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, GRAVE),
    (63941210, DECK),
    (29726552, DECK),
    (55063751, DECK),
    (26202165, MZONE),
    (26202165, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_DESTROY + CATEGORY_SPECIAL_SUMMON);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_count_limit_full(duel, 1, ID, EFFECT_COUNT_CODE_OATH);
    e1.set_target(duel, target);
    e1.set_operation(duel, activate);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_TOHAND + CATEGORY_SEARCH);
    e2.set_type(duel, EFFECT_TYPE_IGNITION);
    e2.set_range(duel, LOCATION_GRAVE);
    e2.set_condition(duel, aux::exccon);
    e2.set_cost(duel, Cost::self_banish);
    e2.set_target(duel, thtg);
    e2.set_operation(duel, thop);
    c.register_effect(duel, e2);
}

fn chkfilter1(c: CardId, duel: &Duel, tp: u8) -> bool {
    c.is_set_card(duel, &[SET_KAIJU])
        && c.is_monster(duel)
        && c.is_has_effect(duel, EFFECT_REVIVE_LIMIT).is_none()
        && duel.is_player_can_special_summon(tp, 0, POS_FACEUP, tp, c)
        && duel.is_existing_matching_card(|tc, duel| chkfilter2(tc, duel, tp, c.get_code(duel)), tp, LOCATION_DECK, 0, 1, None)
}

fn chkfilter2(c: CardId, duel: &Duel, tp: u8, cd: u32) -> bool {
    c.is_set_card(duel, &[SET_KAIJU])
        && c.is_monster(duel)
        && !c.is_code(duel, cd)
        && c.is_has_effect(duel, EFFECT_REVIVE_LIMIT).is_none()
        && duel.is_player_can_special_summon(tp, 0, POS_FACEUP, 1 - tp, c)
}

fn filter1(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_set_card(duel, &[SET_KAIJU])
        && c.is_monster(duel)
        && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
        && duel.is_existing_matching_card(|tc, duel| filter2(tc, duel, e, tp, c.get_code(duel)), tp, LOCATION_DECK, 0, 1, None)
}

fn filter2(c: CardId, duel: &Duel, e: EffectId, tp: u8, cd: u32) -> bool {
    c.is_set_card(duel, &[SET_KAIJU])
        && c.is_monster(duel)
        && !c.is_code(duel, cd)
        && c.is_can_be_special_summoned_full(duel, e, 0, tp, false, false, POS_FACEUP_ATTACK, 1 - tp, 0xff)
}

fn target(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.is_player_affected_by_effect(tp, CARD_BLUEEYES_SPIRIT).is_none()
            && duel.get_field_group_count(tp, LOCATION_MZONE, LOCATION_MZONE) > 0
            && duel.get_location_count(tp, LOCATION_MZONE) > -(duel.get_field_group_count(tp, LOCATION_MZONE, 0) as i32)
            && duel.get_location_count(1 - tp, LOCATION_MZONE) > -(duel.get_field_group_count(1 - tp, LOCATION_MZONE, 0) as i32)
            && duel.is_existing_matching_card(|c, duel| chkfilter1(c, duel, tp), tp, LOCATION_DECK, 0, 1, None));
    }
    let g = duel.get_field_group(tp, LOCATION_MZONE, LOCATION_MZONE);
    duel.set_operation_info(0, CATEGORY_DESTROY, Some(g), g.len() as u8, 0, 0);
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 2, tp, LOCATION_DECK as i32);
    Ok(false)
}

fn activate(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let dg = duel.get_field_group(tp, LOCATION_MZONE, LOCATION_MZONE);
    if duel.destroy(dg, REASON_EFFECT)? == 0 {
        return Ok(());
    }
    let sg = duel.get_matching_group(|c, duel| filter1(c, duel, e, tp), tp, LOCATION_DECK, 0, None);
    if sg.is_empty()
        || duel.is_player_affected_by_effect(tp, CARD_BLUEEYES_SPIRIT).is_some()
        || duel.get_location_count(tp, LOCATION_MZONE) <= 0
        || duel.get_location_count(1 - tp, LOCATION_MZONE) <= 0
    {
        return Ok(());
    }
    duel.break_effect()?;
    duel.hint(HINT_SELECTMSG, tp, aux::stringid(ID, 2));
    let g1 = sg.select(duel, tp, 1, 1, None)?;
    let Some(tc1) = g1.get_first() else { return Ok(()) };
    duel.hint(HINT_SELECTMSG, tp, aux::stringid(ID, 3));
    let code = tc1.get_code(duel);
    let g2 = duel.select_matching_card(tp, |c, duel| filter2(c, duel, e, tp, code), tp, LOCATION_DECK, 0, 1, 1, None)?;
    let Some(tc2) = g2.get_first() else { return Ok(()) };
    duel.special_summon_step(tc1, 0, tp, tp, false, false, POS_FACEUP_ATTACK)?;
    duel.special_summon_step(tc2, 0, tp, 1 - tp, false, false, POS_FACEUP_ATTACK)?;
    let e1 = duel.create_effect(e.get_handler(duel));
    e1.set_description(duel, 3313);
    e1.set_property(duel, EFFECT_FLAG_CLIENT_HINT);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_code(duel, EFFECT_CANNOT_CHANGE_POSITION);
    e1.set_reset(duel, RESET_EVENT | RESETS_STANDARD);
    tc1.register_effect(duel, e1);
    let e2 = e1.clone_effect(duel);
    tc2.register_effect(duel, e2);
    let e3 = e1.clone_effect(duel);
    e3.set_description(duel, 3200);
    e3.set_code(duel, EFFECT_MUST_ATTACK);
    tc1.register_effect(duel, e3);
    let e4 = e3.clone_effect(duel);
    tc2.register_effect(duel, e4);
    duel.special_summon_complete()?;
    Ok(())
}

fn thfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_KAIJU]) && c.is_monster(duel) && c.is_able_to_hand(duel)
}

fn thtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    stock::exists_target(duel, a, chk, thfilter, LOCATION_DECK, CATEGORY_TOHAND)
}

fn thop(duel: &mut Duel, a: &Args) -> Script<()> {
    stock::search(duel, a, thfilter, LOCATION_DECK)
}
