// Branded in High Spirits
use crate::prelude::*;
use crate::testing::{Check, DECK, EXTRA, GRAVE, HAND};

const ID: u32 = 29948294;

pub const LISTED_NAMES: &[u32] = &[CARD_ALBAZ];

// Red-Eyes Dark Dragoon (Spellcaster Fusion, DEF 2500) in the Extra Deck to send, a Spellcaster in the hand to reveal and
// discard, Fallen of Albaz in the Deck to add, and a copy in the Graveyard to return.
pub const SUPPORT: &[(u32, u16)] = &[(ID, HAND), (ID, GRAVE), (37818794, EXTRA), (15025844, HAND), (CARD_ALBAZ, DECK)];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_TOGRAVE + CATEGORY_HANDES + CATEGORY_TOHAND + CATEGORY_SEARCH);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_hint_timing(duel, 0, TIMINGS_CHECK_MONSTER_E);
    e1.set_count_limit(duel, 1, ID);
    e1.set_target(duel, target);
    e1.set_operation(duel, activate);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_TOHAND);
    e2.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e2.set_code(duel, EVENT_PHASE + PHASE_END);
    e2.set_range(duel, LOCATION_GRAVE);
    e2.set_count_limit(duel, 1, ID);
    e2.set_condition(duel, |duel, a| duel.get_flag_effect(a.tp, ID) > 0);
    e2.set_target(duel, thtg);
    e2.set_operation(duel, thop);
    c.register_effect(duel, e2);
    aux::global_check(duel, ID, |duel| {
        let ge1 = duel.create_effect(c);
        ge1.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_CONTINUOUS);
        ge1.set_code(duel, EVENT_TO_GRAVE);
        ge1.set_operation(duel, checkop);
        duel.register_effect(ge1, 0);
    });
}

fn checkop(duel: &mut Duel, a: &Args) -> Script<()> {
    for tc in a.eg.unwrap_or_default() {
        if tc.is_type(duel, TYPE_FUSION) {
            duel.register_flag_effect(tc.get_controler(duel), ID, RESET_PHASE | PHASE_END, 0, 1);
        }
    }
    Ok(())
}

fn tgfilter(c: CardId, duel: &Duel, race: u64) -> bool {
    c.is_level(duel, &[8])
        && c.is_type(duel, TYPE_FUSION)
        && (c.is_attack(duel, &[2500]) || c.is_defense(duel, &[2500]))
        && c.is_race(duel, race)
        && c.is_able_to_grave(duel)
}

fn revfilter(c: CardId, duel: &Duel, tp: u8) -> bool {
    let race = c.get_race(duel);
    c.is_monster(duel)
        && !c.is_public(duel)
        && duel.is_existing_matching_card(|c, duel| tgfilter(c, duel, race), tp, LOCATION_EXTRA, 0, 1, None)
}

fn target(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(|c, duel| revfilter(c, duel, tp), tp, LOCATION_HAND, 0, 1, None));
    }
    duel.set_operation_info(0, CATEGORY_TOGRAVE, None, 1, tp, LOCATION_EXTRA as i32);
    duel.set_possible_operation_info(0, CATEGORY_TOHAND, None, 1, tp, LOCATION_DECK as i32);
    Ok(false)
}

fn thfilter(c: CardId, duel: &Duel) -> bool {
    (c.is_code(duel, CARD_ALBAZ) || (c.is_monster(duel) && c.lists_code(duel, CARD_ALBAZ))) && c.is_able_to_hand(duel)
}

fn activate(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_CONFIRM);
    let rg = duel.select_matching_card(tp, |c, duel| revfilter(c, duel, tp), tp, LOCATION_HAND, 0, 1, 1, None)?;
    let Some(cc) = rg.get_first() else { return Ok(()) };
    duel.confirm_cards(1 - tp, rg)?;
    duel.shuffle_hand(tp)?;
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TOGRAVE);
    let race = cc.get_race(duel);
    let g = duel.select_matching_card(tp, |c, duel| tgfilter(c, duel, race), tp, LOCATION_EXTRA, 0, 1, 1, None)?;
    if let Some(tc) = g.get_first()
        && duel.send_to_grave(g, REASON_EFFECT)? > 0
        && tc.is_location(duel, LOCATION_GRAVE)
        && cc.is_discardable_full(duel, REASON_EFFECT, None)
        && duel.is_existing_matching_card(thfilter, tp, LOCATION_DECK, 0, 1, None)
        && duel.select_yes_no(tp, aux::stringid(ID, 2))?
    {
        duel.break_effect()?;
        if duel.send_to_grave(cc, REASON_EFFECT | REASON_DISCARD)? > 0 {
            duel.hint(HINT_SELECTMSG, tp, HINTMSG_ATOHAND);
            let hg = duel.select_matching_card(tp, thfilter, tp, LOCATION_DECK, 0, 1, 1, None)?;
            if !hg.is_empty() {
                duel.send_to_hand(hg, PLAYER_NONE, REASON_EFFECT)?;
                duel.confirm_cards(1 - tp, hg)?;
            }
        }
    }
    Ok(())
}

fn thtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let c = a.e.get_handler(duel);
    if chk == 0 {
        return Ok(c.is_able_to_hand(duel));
    }
    duel.set_operation_info(0, CATEGORY_TOHAND, Some(c.into()), 1, 0, 0);
    Ok(false)
}

fn thop(duel: &mut Duel, a: &Args) -> Script<()> {
    let c = a.e.get_handler(duel);
    if c.is_relate_to_effect(duel, a.e) {
        duel.send_to_hand(c, PLAYER_NONE, REASON_EFFECT)?;
    }
    Ok(())
}
