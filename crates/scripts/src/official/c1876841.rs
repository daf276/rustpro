// Morphtronic Scannen
use crate::prelude::*;
use crate::testing::{Check, DECK, GRAVE, HAND};

const ID: u32 = 1876841;

pub const LISTED_SERIES: &[u16] = &[SET_MORPHTRONIC];
// Morphtronic Lantron to banish from the hand and to retrieve from the Grave, Morphtronic Accelerator in the Deck to search.
pub const SUPPORT: &[(u32, u16)] = &[
    (76865611, HAND),
    (76865611, HAND),
    (76865611, GRAVE),
    (44424095, DECK),
    (44424095, DECK),
];
pub const CHECKS: &[Check] = &[
    Check::SpecialSummoned,
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_code(duel, EFFECT_SPSUMMON_PROC);
    e1.set_property(duel, EFFECT_FLAG_UNCOPYABLE);
    e1.set_range(duel, LOCATION_HAND);
    e1.set_condition(duel, spcon);
    e1.set_target(duel, sptg);
    e1.set_operation(duel, spop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 0));
    e2.set_category(duel, CATEGORY_SEARCH + CATEGORY_TOHAND + CATEGORY_TODECK);
    e2.set_type(duel, EFFECT_TYPE_IGNITION);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_count_limit(duel, 1, 0);
    e2.set_condition(duel, |duel, a| a.e.get_handler(duel).is_attack_pos(duel));
    e2.set_target(duel, target_a);
    e2.set_operation(duel, operation_a);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 1));
    e3.set_category(duel, CATEGORY_TOHAND + CATEGORY_TODECK);
    e3.set_type(duel, EFFECT_TYPE_IGNITION);
    e3.set_range(duel, LOCATION_MZONE);
    e3.set_count_limit(duel, 1, 0);
    e3.set_condition(duel, |duel, a| a.e.get_handler(duel).is_defense_pos(duel));
    e3.set_target(duel, target_d);
    e3.set_operation(duel, operation_d);
    c.register_effect(duel, e3);
}

fn cfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_MORPHTRONIC]) && c.is_monster(duel) && c.is_able_to_remove_as_cost(duel)
}

fn spcon(duel: &Duel, a: &Args) -> bool {
    let Some(c) = a.c else { return true };
    let tp = c.get_controler(duel);
    duel.get_location_count(tp, LOCATION_MZONE) > 0 && duel.is_existing_matching_card(cfilter, tp, LOCATION_HAND, 0, 1, Some(c.into()))
}

fn sptg(duel: &mut Duel, a: &Args, _: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_REMOVE);
    let g = duel.select_matching_card_full(tp, cfilter, tp, LOCATION_HAND, 0, 1, 1, true, a.c.map(Into::into))?;
    if g.is_empty() {
        return Ok(false);
    }
    e.set_label_object_group(duel, Some(g));
    Ok(true)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    if let Some(g) = a.e.get_label_object_group(duel) {
        duel.remove(g, POS_FACEUP, REASON_COST)?;
    }
    Ok(())
}

fn thdfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_MORPHTRONIC]) && c.is_spell_trap(duel) && c.is_able_to_hand(duel)
}

fn target_a(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(thdfilter, tp, LOCATION_DECK, 0, 1, None));
    }
    duel.set_operation_info(0, CATEGORY_TOHAND, None, 1, tp, LOCATION_DECK as i32);
    duel.set_operation_info(0, CATEGORY_TODECK, None, 1, tp, LOCATION_HAND as i32);
    Ok(false)
}

fn return_to_deck(duel: &mut Duel, tp: u8) -> Script<()> {
    duel.break_effect()?;
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TODECK);
    let dg = duel.select_matching_card(tp, CardId::is_able_to_deck, tp, LOCATION_HAND, 0, 1, 1, None)?;
    duel.send_to_deck(dg, PLAYER_NONE, SEQ_DECKTOP, REASON_EFFECT)?;
    Ok(())
}

fn operation_a(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_ATOHAND);
    let g = duel.select_matching_card(tp, thdfilter, tp, LOCATION_DECK, 0, 1, 1, None)?;
    if let Some(tc) = g.get_first()
        && duel.send_to_hand(g, PLAYER_NONE, REASON_EFFECT)? > 0
        && tc.is_location(duel, LOCATION_HAND)
    {
        duel.confirm_cards(1 - tp, g)?;
        duel.shuffle_hand(tp)?;
        duel.shuffle_deck(tp)?;
        return_to_deck(duel, tp)?;
    }
    Ok(())
}

fn thgfilter(c: CardId, duel: &Duel) -> bool {
    c.is_monster(duel) && c.is_set_card(duel, &[SET_MORPHTRONIC]) && c.is_level_below(duel, 4) && c.is_able_to_hand(duel)
}

fn target_d(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(thgfilter, tp, LOCATION_GRAVE, 0, 1, None));
    }
    duel.set_operation_info(0, CATEGORY_TOHAND, None, 1, tp, LOCATION_GRAVE as i32);
    duel.set_operation_info(0, CATEGORY_TODECK, None, 1, tp, LOCATION_HAND as i32);
    Ok(false)
}

fn operation_d(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_ATOHAND);
    let g = duel.select_matching_card(tp, thgfilter, tp, LOCATION_GRAVE, 0, 1, 1, None)?;
    if let Some(tc) = g.get_first()
        && duel.send_to_hand(g, PLAYER_NONE, REASON_EFFECT)? > 0
        && tc.is_location(duel, LOCATION_HAND)
    {
        duel.confirm_cards(1 - tp, g)?;
        duel.shuffle_hand(tp)?;
        return_to_deck(duel, tp)?;
    }
    Ok(())
}
