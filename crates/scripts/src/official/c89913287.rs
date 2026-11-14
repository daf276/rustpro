// Adamancipator Risen - Tiamite
use super::c44035031::Procedure;
use crate::prelude::*;
use crate::testing::{Check, DECK, EXTRA, GRAVE, MZONE};

const ID: u32 = 89913287;

pub const LISTED_SERIES: &[u16] = &[SET_ADAMANCIPATOR];
// In the Extra Deck, Synchro Summoned with Angel Trumpeter (a Level 4 Tuner) and Doron (Level 2), with Adamancipator Mates and
// Rock monsters in the Deck and a DARK monster (Archfiend Soldier) in the Graveyard.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (87979586, MZONE),
    (87979586, MZONE),
    (756652, MZONE),
    (756652, MZONE),
    (30068809, DECK),
    (30068809, DECK),
    (48519867, DECK),
    (85914562, DECK),
    (11302671, DECK),
    (49881766, GRAVE),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Synchro::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_TOHAND + CATEGORY_SEARCH);
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_count_limit(duel, 1, (ID, 0));
    e1.set_condition(duel, |duel, a| {
        duel.is_existing_matching_card(|c, duel| c.is_attribute(duel, ATTRIBUTE_DARK), a.tp, LOCATION_GRAVE, 0, 1, None)
    });
    e1.set_target(duel, thtg);
    e1.set_operation(duel, thop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_TOHAND);
    e2.set_type(duel, EFFECT_TYPE_QUICK_O);
    e2.set_code(duel, EVENT_CHAINING);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_condition(duel, |duel, a| {
        a.ep == 1 - a.tp && chain::is_triggering_location(duel, a.ev as u8, LOCATION_MZONE)
    });
    e2.set_target(duel, excavtg);
    e2.set_operation(duel, excavop);
    c.register_effect(duel, e2);
}

fn thfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_ADAMANCIPATOR]) && c.is_spell_trap(duel) && c.is_able_to_hand(duel)
}

fn thtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    stock::exists_target(duel, a, chk, thfilter, LOCATION_DECK, CATEGORY_TOHAND)
}

fn thop(duel: &mut Duel, a: &Args) -> Script<()> {
    stock::search(duel, a, thfilter, LOCATION_DECK)
}

fn excavtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.get_field_group_count(tp, LOCATION_DECK, 0) >= 5);
    }
    duel.set_possible_operation_info(0, CATEGORY_TOHAND, None, 1, 1 - tp, LOCATION_ONFIELD as i32);
    Ok(false)
}

fn excavop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    if duel.get_field_group_count(tp, LOCATION_DECK, 0) == 0 {
        return Ok(());
    }
    duel.confirm_decktop(tp, 5)?;
    let excavg = duel.get_decktop_group(tp, 5);
    let excav_count = excavg.len();
    let excav_rock_count = excavg.filter_count(duel, |c, duel| c.is_race(duel, RACE_ROCK), None);
    let g = duel.get_matching_group(CardId::is_able_to_hand, tp, 0, LOCATION_ONFIELD, None);
    if excav_rock_count > 0 && !g.is_empty() && duel.select_yes_no(tp, aux::stringid(ID, 2))? {
        let return_count = g.len().min(excav_rock_count);
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_RTOHAND);
        let sg = g.select(duel, tp, 1, return_count as u8, None)?;
        if !sg.is_empty() {
            duel.hint_selection(sg);
            duel.send_to_hand(sg, PLAYER_NONE, REASON_EFFECT)?;
            let og = duel
                .get_operated_group()
                .filter(duel, |c, duel| c.is_location(duel, LOCATION_HAND), None);
            if !og.is_empty() {
                if og.is_exists(duel, |c, duel| c.is_controler(duel, tp), 1, None) {
                    duel.shuffle_hand(tp)?;
                }
                if og.is_exists(duel, |c, duel| c.is_controler(duel, 1 - tp), 1, None) {
                    duel.shuffle_hand(1 - tp)?;
                }
            }
        }
    }
    if duel.get_field_group_count(tp, LOCATION_DECK, 0) > excav_count {
        duel.move_to_deck_bottom_count(excav_count, tp)?;
    }
    duel.sort_deckbottom(tp, tp, excav_count as u16)?;
    Ok(())
}
