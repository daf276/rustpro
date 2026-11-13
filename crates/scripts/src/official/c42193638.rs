// Vaalmonica Versare
use crate::prelude::*;
use crate::testing::{Check, DECK, HAND, PZONE};

const ID: u32 = 42193638;

pub const LISTED_NAMES: &[u32] = &[ID];

// In the hand with Angello and Dimonno in the Pendulum Zone, and Vaalmonica cards in the Deck to excavate or send.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (3048768, PZONE),
    (30432463, PZONE),
    (3048768, DECK),
    (3048768, DECK),
    (30432463, DECK),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(
        duel,
        CATEGORY_RECOVER + CATEGORY_TOHAND + CATEGORY_SEARCH + CATEGORY_DAMAGE + CATEGORY_TOGRAVE,
    );
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_count_limit_full(duel, 1, ID, EFFECT_COUNT_CODE_OATH);
    e1.set_target(duel, target);
    e1.set_operation(duel, activate);
    c.register_effect(duel, e1);
}

fn target(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(true);
    }
    duel.set_possible_operation_info(0, CATEGORY_RECOVER, None, 0, tp, 500);
    duel.set_possible_operation_info(0, CATEGORY_TOHAND, None, 1, tp, LOCATION_DECK as i32);
    duel.set_possible_operation_info(0, CATEGORY_DAMAGE, None, 0, 1 - tp, 500);
    duel.set_possible_operation_info(0, CATEGORY_TOGRAVE, None, 1, tp, LOCATION_DECK as i32);
    Ok(false)
}

fn tgfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_VAALMONICA]) && !c.is_code(duel, ID) && c.is_able_to_grave(duel)
}

// The option of Versare, Scelta and Intonare: the one passed by Angello or Dimonno, else a choice by the player with a Vaalmonica in the Pendulum Zone.
pub fn select_option(duel: &mut Duel, a: &Args, id: u32) -> Script<Option<usize>> {
    let tp = a.tp;
    if a.extra != 0 {
        return Ok(Some(usize::from(a.extra)));
    }
    let vaalmonica = duel.is_existing_matching_card(|c, duel| c.is_set_card(duel, &[SET_VAALMONICA]), tp, LOCATION_PZONE, 0, 1, None);
    let (sel_player, offset) = if vaalmonica { (tp, 0) } else { (1 - tp, 2) };
    duel.select_effect(
        sel_player,
        &[(true, aux::stringid(id, 1 + offset)), (true, aux::stringid(id, 2 + offset))],
    )
}

fn activate(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    let op = select_option(duel, a, ID)?;
    if op == Some(1) {
        let g = duel.get_matching_group(|c, duel| c.is_set_card(duel, &[SET_VAALMONICA]), tp, LOCATION_DECK, 0, None);
        if duel.recover(tp, 500, REASON_EFFECT)? > 0 && !g.is_empty() && duel.select_yes_no(tp, aux::stringid(ID, 5))? {
            let Some((spcard, seq)) = g.get_max_group(duel, |c, duel| i64::from(c.get_sequence(duel))) else {
                return Ok(());
            };
            let Some(spcard) = spcard.get_first() else { return Ok(()) };
            duel.break_effect()?;
            duel.confirm_decktop(tp, duel.get_field_group_count(tp, LOCATION_DECK, 0).saturating_sub(seq as usize))?;
            if spcard.is_able_to_hand(duel) {
                duel.send_to_hand(spcard, PLAYER_NONE, REASON_EFFECT)?;
                duel.confirm_cards(1 - tp, spcard)?;
            } else {
                duel.send_to_grave(spcard, REASON_RULE)?;
            }
        }
    } else if op == Some(2) {
        let g = duel.get_matching_group(tgfilter, tp, LOCATION_DECK, 0, None);
        if duel.damage(tp, 500, REASON_EFFECT)? > 0 && !g.is_empty() && duel.select_yes_no(tp, aux::stringid(ID, 6))? {
            duel.hint(HINT_SELECTMSG, tp, HINTMSG_TOGRAVE);
            let sg = g.select(duel, tp, 1, 1, None)?;
            duel.break_effect()?;
            duel.send_to_grave(sg, REASON_EFFECT)?;
        }
    }
    Ok(())
}
