// Where Arf Thou?
use crate::prelude::*;
use crate::testing::{Check, DECK, HAND, MZONE};

const ID: u32 = 51405049;

// In the hand, with a Level 1 Normal Monster, Key Mace, on the field and two more in the Deck.
pub const SUPPORT: &[(u32, u16)] = &[(ID, HAND), (1929294, MZONE), (1929294, DECK), (1929294, DECK)];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(0)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_category(duel, CATEGORY_TOHAND + CATEGORY_SEARCH);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_condition(duel, condition);
    e1.set_target(duel, target);
    e1.set_operation(duel, activate);
    c.register_effect(duel, e1);
}

fn cfilter(c: CardId, duel: &Duel) -> bool {
    c.is_faceup(duel) && c.get_level(duel) == 1
}

fn condition(duel: &Duel, a: &Args) -> bool {
    duel.is_existing_matching_card(cfilter, a.tp, LOCATION_MZONE, 0, 1, None)
}

fn filter(c: CardId, duel: &Duel) -> bool {
    c.get_level(duel) == 1 && c.is_able_to_hand(duel)
}

fn target(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(filter, tp, LOCATION_DECK, 0, 1, None));
    }
    duel.set_operation_info(0, CATEGORY_TOHAND, None, 1, tp, LOCATION_DECK as i32);
    Ok(false)
}

fn activate(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_ATOHAND);
    let g = duel.select_matching_card(tp, filter, tp, LOCATION_DECK, 0, 1, 1, None)?;
    let Some(tc) = g.get_first() else {
        return Ok(());
    };
    duel.send_to_hand(g, PLAYER_NONE, REASON_EFFECT)?;
    duel.confirm_cards(1 - tp, g)?;
    let handler = a.e.get_handler(duel);
    let e1 = duel.create_effect(handler);
    e1.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_CONTINUOUS);
    e1.set_code(duel, EVENT_SUMMON_SUCCESS);
    e1.set_operation(duel, regop);
    e1.set_label(duel, tc.get_code(duel).into());
    e1.set_reset(duel, RESET_PHASE | PHASE_END);
    duel.register_effect(e1, tp);
    let e2 = duel.create_effect(handler);
    e2.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_CONTINUOUS);
    e2.set_code(duel, EVENT_PHASE + PHASE_END);
    e2.set_count_limit(duel, 1, 0);
    e2.set_condition(duel, damcon);
    e2.set_operation(duel, damop);
    e2.set_reset(duel, RESET_PHASE | PHASE_END);
    e2.set_label_object(duel, Some(e1));
    duel.register_effect(e2, tp);
    Ok(())
}

fn regop(duel: &mut Duel, a: &Args) -> Script<()> {
    if a.e.get_label(duel) == 0 {
        return Ok(());
    }
    if let Some(tc) = a.eg.and_then(|eg| eg.get_first())
        && tc.is_code(duel, a.e.get_label(duel) as u32)
    {
        a.e.set_label(duel, 0);
    }
    Ok(())
}

fn damcon(duel: &Duel, a: &Args) -> bool {
    a.e.get_label_object(duel).is_some_and(|e1| e1.get_label(duel) != 0)
}

fn damop(duel: &mut Duel, a: &Args) -> Script<()> {
    duel.damage(a.tp, 2000, REASON_EFFECT)?;
    Ok(())
}
