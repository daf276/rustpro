// Ghostrick Siren
use super::c80885284::{RESET_FLAG_STANDARD_PHASE_END, posop, postg, sumcon};
use crate::prelude::*;
use crate::testing::{Check, DECK, HAND, MZONE, OPPONENT};

const ID: u32 = 64804316;
const GHOSTRICK_JIANGSHI: u32 = 80885284;
const GHOSTRICK_SCARE: u32 = 86516889;
const ELFNOTE_LUCINA: u32 = 13597785;

// A Ghostrick monster on the field to Normal Summon from the hand, face-down copies with an attacker of the opponent's to flip
// them, and Ghostrick Jiangshi and Scare in the Deck to send to the Graveyard and search.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, MZONE),
    (ID, MZONE),
    (GHOSTRICK_JIANGSHI, MZONE),
    (GHOSTRICK_JIANGSHI, DECK),
    (GHOSTRICK_SCARE, DECK),
    (ELFNOTE_LUCINA, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_code(duel, EFFECT_CANNOT_SUMMON);
    e1.set_condition(duel, sumcon);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 0));
    e2.set_category(duel, CATEGORY_POSITION + CATEGORY_SET);
    e2.set_type(duel, EFFECT_TYPE_IGNITION);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_target(duel, postg::<ID, RESET_FLAG_STANDARD_PHASE_END>);
    e2.set_operation(duel, posop);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 1));
    e3.set_category(
        duel,
        CATEGORY_TOGRAVE + CATEGORY_DECKDES + CATEGORY_TOHAND + CATEGORY_SEARCH + CATEGORY_POSITION + CATEGORY_SET,
    );
    e3.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_F);
    e3.set_code(duel, EVENT_SUMMON_SUCCESS);
    e3.set_target(duel, gytg);
    e3.set_operation(duel, gyop);
    c.register_effect(duel, e3);
    let e4 = e3.clone_effect(duel);
    e4.set_code(duel, EVENT_FLIP);
    c.register_effect(duel, e4);
}

fn gytg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(true);
    }
    let g = duel.get_decktop_group(tp, 2);
    duel.set_operation_info(0, CATEGORY_TOGRAVE, Some(g), g.len() as u8, tp, LOCATION_DECK as i32);
    duel.set_possible_operation_info(0, CATEGORY_TOHAND, None, 1, tp, LOCATION_DECK as i32);
    duel.set_possible_operation_info(0, CATEGORY_POSITION, None, 1, 1 - tp, POS_FACEDOWN_DEFENSE as i32);
    Ok(false)
}

fn thfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_GHOSTRICK]) && c.is_spell_trap(duel) && c.is_able_to_hand(duel)
}

fn posfilter(c: CardId, duel: &Duel) -> bool {
    c.is_faceup(duel) && c.is_type(duel, TYPE_EFFECT) && c.is_can_turn_set(duel)
}

fn gyop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    if duel.discard_deck(tp, 2, REASON_EFFECT)? != 2 {
        return Ok(());
    }
    let og = duel.get_operated_group().filter(
        duel,
        |c, duel| c.is_set_card(duel, &[SET_GHOSTRICK]) && c.is_location(duel, LOCATION_GRAVE),
        None,
    );
    if og.is_empty() {
        return Ok(());
    }
    let g1 = duel.get_matching_group(thfilter, tp, LOCATION_DECK, 0, None);
    let g2 = duel.get_matching_group(posfilter, tp, 0, LOCATION_MZONE, None);
    let (b1, b2) = (!g1.is_empty(), !g2.is_empty());
    if !(b1 || b2) || !duel.select_yes_no(tp, aux::stringid(ID, 2))? {
        return Ok(());
    }
    let Some(op) = duel.select_effect(tp, &[(b1, aux::stringid(ID, 3)), (b2, aux::stringid(ID, 4))])? else {
        return Ok(());
    };
    duel.break_effect()?;
    if op == 1 {
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_ATOHAND);
        let sg = g1.select(duel, tp, 1, 1, None)?;
        duel.send_to_hand(sg, tp, REASON_EFFECT)?;
        duel.confirm_cards(1 - tp, sg)?;
    } else if op == 2 {
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_POSCHANGE);
        let sg = g2.select(duel, tp, 1, 1, None)?;
        duel.hint_selection_full(sg, true);
        duel.change_position(sg, POS_FACEDOWN_DEFENSE)?;
    }
    Ok(())
}
