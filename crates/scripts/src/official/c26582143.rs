// Gladiator Beast Gistel
use crate::prelude::*;
use crate::testing::{Check, DECK, HAND, MZONE, OPPONENT};

const ID: u32 = 26582143;

pub const LISTED_SERIES: &[u16] = &[SET_GLADIATOR_BEAST, SET_GLADIATOR];
pub const LISTED_NAMES: &[u32] = &[ID];
// In the hand with Gladiator Beast Hoplomus to reveal, on the field after battle with Hoplomus in the Deck, and Gladiator Beast
// Charge in the Deck to add.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, MZONE),
    (ID, MZONE),
    (4253484, HAND),
    (4253484, HAND),
    (4253484, DECK),
    (4253484, DECK),
    (16990348, DECK),
    (16990348, DECK),
    (27125110, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
    Check::ResolvesEffect(aux::stringid(ID, 2)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e1.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e1.set_code(duel, EVENT_PHASE | PHASE_BATTLE);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_condition(duel, |duel, a| a.e.get_handler(duel).get_battled_group_count(duel) > 0);
    e1.set_cost(duel, Cost::self_to_deck);
    e1.set_target(duel, |duel, a, chk, _| {
        super::c57731460::sptg(duel, a, chk, |c, duel| deckspfilter(c, duel, a.e, a.tp))
    });
    e1.set_operation(duel, |duel, a| {
        super::c57731460::spop(duel, a, |c, duel| deckspfilter(c, duel, a.e, a.tp), 100, true, 1)
    });
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e2.set_type(duel, EFFECT_TYPE_IGNITION);
    e2.set_range(duel, LOCATION_HAND);
    e2.set_count_limit(duel, 1, ID);
    e2.set_cost(duel, handspcost);
    e2.set_target(duel, handsptg);
    e2.set_operation(duel, handspop);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 2));
    e3.set_category(duel, CATEGORY_TOHAND + CATEGORY_SEARCH);
    e3.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e3.set_property(duel, EFFECT_FLAG_DELAY);
    e3.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e3.set_count_limit(duel, 1, (ID, 1));
    e3.set_condition(duel, aux::gbspcon);
    e3.set_target(duel, thtg);
    e3.set_operation(duel, thop);
    c.register_effect(duel, e3);
}

fn deckspfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_set_card(duel, &[SET_GLADIATOR_BEAST]) && c.is_can_be_special_summoned(duel, e, 100, tp, false, false) && !c.is_code(duel, ID)
}

fn handspcostfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_set_card(duel, &[SET_GLADIATOR_BEAST]) && !c.is_public(duel) && c.is_can_be_special_summoned(duel, e, 100, tp, false, false)
}

fn handspcost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    let filter = |rc, duel: &Duel| handspcostfilter(rc, duel, e, tp);
    if chk == 0 {
        return Ok(!c.is_public(duel) && duel.is_existing_matching_card(filter, tp, LOCATION_HAND, 0, 1, Some(c.into())));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_CONFIRM);
    let rc = duel.select_matching_card(tp, filter, tp, LOCATION_HAND, 0, 1, 1, Some(c.into()))?;
    duel.confirm_cards(1 - tp, rc.union(c.into()))?;
    e.set_label_object_group(duel, Some(rc));
    duel.shuffle_hand(tp)?;
    Ok(false)
}

fn handsptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if chk == 0 {
        return Ok(duel.is_player_affected_by_effect(tp, CARD_BLUEEYES_SPIRIT).is_none()
            && duel.get_location_count(tp, LOCATION_MZONE) >= 2
            && c.is_can_be_special_summoned(duel, e, 100, tp, false, false));
    }
    let g = e.get_label_object_group(duel).unwrap_or_default().union(c.into());
    duel.set_target_card(g);
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, Some(g), 2, tp, 0);
    Ok(false)
}

fn handspop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    if duel.is_player_affected_by_effect(tp, CARD_BLUEEYES_SPIRIT).is_some() || duel.get_location_count(tp, LOCATION_MZONE) < 2 {
        return Ok(());
    }
    let sg = duel.get_target_cards(e).unwrap_or_default();
    if sg.len() == 2
        && sg.filter_count(duel, |c, duel| c.is_can_be_special_summoned(duel, e, 100, tp, false, false), None) == 2
        && duel.special_summon(sg, 100, tp, tp, false, false, POS_FACEUP)? == 2
    {
        for sc in sg {
            let code = sc.get_original_code(duel);
            sc.register_flag_effect(duel, code, RESET_EVENT | RESETS_STANDARD_DISABLE, 0, 1);
        }
    }
    Ok(())
}

fn thfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_GLADIATOR]) && c.is_spell_trap(duel) && c.is_able_to_hand(duel)
}

fn thtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    stock::exists_target(duel, a, chk, thfilter, LOCATION_DECK, CATEGORY_TOHAND)
}

fn thop(duel: &mut Duel, a: &Args) -> Script<()> {
    stock::search(duel, a, thfilter, LOCATION_DECK)
}
