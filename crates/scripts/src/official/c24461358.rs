// Ragged Records of Rites
use crate::prelude::*;
use crate::testing::{Check, DECK, GRAVE, HAND, MZONE};

const ID: u32 = 24461358;

// In the hand, revealing Black Magic Ritual to add Magician of Black Chaos from the Deck, and in the GY with Magician of
// Black Chaos on the field to leave it by effect.
pub const SUPPORT: &[(u32, u16)] = &[(ID, HAND), (76792184, HAND), (30208479, DECK), (ID, GRAVE), (30208479, MZONE)];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_TOHAND + CATEGORY_SEARCH);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_count_limit(duel, 1, (ID, 0));
    e1.set_target(duel, target);
    e1.set_operation(duel, activate);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_TOHAND);
    e2.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e2.set_property_full(duel, EFFECT_FLAG_DELAY, EFFECT_FLAG2_CHECK_SIMULTANEOUS);
    e2.set_code(duel, EVENT_LEAVE_FIELD);
    e2.set_range(duel, LOCATION_GRAVE);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_condition(duel, selfthcon);
    e2.set_target(duel, selfthtg);
    e2.set_operation(duel, stock::self_to_hand);
    c.register_effect(duel, e2);
}

fn thfilter(c: CardId, duel: &Duel, code: u32) -> bool {
    c.is_monster(duel) && c.lists_code(duel, code) && c.is_able_to_hand(duel)
}

fn revealfilter(c: CardId, duel: &Duel, tp: u8) -> bool {
    c.is_ritual_spell(duel)
        && !c.is_public(duel)
        && duel.is_existing_matching_card(|tc, duel| thfilter(tc, duel, c.get_code(duel)), tp, LOCATION_DECK, 0, 1, None)
}

fn target(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(|c, duel| revealfilter(c, duel, tp), tp, LOCATION_HAND | LOCATION_DECK, 0, 1, None));
    }
    duel.set_operation_info(0, CATEGORY_TOHAND, None, 1, tp, LOCATION_DECK as i32);
    Ok(false)
}

fn rescon(duel: &Duel, sg: CardSet, _: CardSet, _: Option<CardId>) -> (bool, bool) {
    let ritual_spell = sg.filter(duel, |c, duel| c.is_ritual_spell(duel), None).get_first();
    let res = ritual_spell.is_some_and(|spell| sg.is_exists(duel, |c, duel| thfilter(c, duel, spell.get_code(duel)), 1, None));
    (res, ritual_spell.is_none())
}

fn activate(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    let mut g = duel.get_matching_group(|c, duel| revealfilter(c, duel, tp), tp, LOCATION_HAND | LOCATION_DECK, 0, None);
    if g.is_empty() {
        return Ok(());
    }
    g.merge(duel.get_matching_group(|c, duel| c.is_monster(duel) && c.is_able_to_hand(duel), tp, LOCATION_DECK, 0, None));
    let sg = aux::select_unselect_group(duel, g, 2, 2, &rescon, tp, aux::stringid(ID, 2))?;
    if sg.len() == 2 {
        let (ritual_spell, monster) = sg.split(duel, |c, duel| c.is_ritual_spell(duel), None);
        duel.confirm_cards(1 - tp, ritual_spell)?;
        duel.send_to_hand(monster, PLAYER_NONE, REASON_EFFECT)?;
        duel.confirm_cards(1 - tp, monster)?;
    }
    Ok(())
}

fn selfthconfilter(c: CardId, duel: &Duel, tp: u8) -> bool {
    c.is_ritual_monster(duel)
        && c.is_previous_position(duel, POS_FACEUP)
        && c.is_previous_controler(duel, tp)
        && c.is_previous_location(duel, LOCATION_MZONE)
        && c.is_reason(duel, REASON_EFFECT)
}

fn selfthcon(duel: &Duel, a: &Args) -> bool {
    let eg = a.eg.unwrap_or_default();
    !eg.is_contains(a.e.get_handler(duel)) && eg.is_exists(duel, |c, duel| selfthconfilter(c, duel, a.tp), 1, None)
}

fn selfthtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (c, tp) = (a.e.get_handler(duel), a.tp);
    if chk == 0 {
        let non_ritual = aux::faceup_filter(|c, duel| !c.is_ritual_monster(duel));
        return Ok(c.is_able_to_hand(duel) && !duel.is_existing_matching_card(non_ritual, tp, LOCATION_MZONE, 0, 1, None));
    }
    duel.set_operation_info(0, CATEGORY_TOHAND, Some(c.into()), 1, tp, 0);
    Ok(false)
}
