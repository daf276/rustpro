// Seventh Ascension
use engine::{Operation, Target};

use crate::prelude::*;
use crate::testing::{Check, DECK, EXTRA, GRAVE, HAND, MZONE, OPPONENT};

const ID: u32 = 23153227;

pub const LISTED_SERIES: &[u16] = &[SET_SEVENTH, SET_BARIANS, SET_RANK_UP_MAGIC];
pub const LISTED_NAMES: &[u32] = &[ID];

// In the GY with The Phantom Knights' Rank-Up-Magic Requiem in the hand to copy, and Extra Deck monsters of the opponent's
// to Summon; Seventh Tachyon and a Seventh Ascension of its own in the Deck to add.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, GRAVE),
    (ID, DECK),
    (7477101, DECK),
    (7477101, DECK),
    (62104532, HAND),
    (62709239, MZONE),
    (62709239, EXTRA | OPPONENT),
    (28781003, EXTRA | OPPONENT),
    (28781003, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::Report(&Check::ResolvesEffect(aux::stringid(ID, 1))),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_TOHAND + CATEGORY_SEARCH);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_count_limit(duel, 1, ID);
    e1.set_target(duel, target);
    e1.set_operation(duel, activate);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_type(duel, EFFECT_TYPE_IGNITION);
    e2.set_range(duel, LOCATION_GRAVE);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_condition(duel, copycon);
    e2.set_cost(duel, copycost);
    e2.set_target(duel, copytg);
    e2.set_operation(duel, copyop);
    c.register_effect(duel, e2);
}

fn filter(c: CardId, duel: &Duel, dct: usize) -> bool {
    (((c.is_set_card(duel, &[SET_SEVENTH]) && !c.is_code(duel, ID)) || c.is_set_card(duel, &[SET_BARIANS])) && c.is_spell_trap(duel)
        || (c.is_set_card(duel, &[SET_RANK_UP_MAGIC]) && c.is_quick_play_spell(duel)))
        && (c.is_able_to_hand(duel) || dct > 1)
}

fn target(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    let dct = duel.get_field_group_count(tp, LOCATION_DECK, 0);
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(|c, duel| filter(c, duel, dct), tp, LOCATION_DECK, 0, 1, None));
    }
    Ok(false)
}

fn activate(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    let dct = duel.get_field_group_count(tp, LOCATION_DECK, 0);
    duel.hint(HINT_SELECTMSG, tp, aux::stringid(ID, 2));
    let g = duel.select_matching_card(tp, |c, duel| filter(c, duel, dct), tp, LOCATION_DECK, 0, 1, 1, None)?;
    if let Some(tc) = g.get_first() {
        aux::to_hand_or_else(
            duel,
            tc,
            tp,
            |_, _| dct > 1,
            |duel| {
                duel.shuffle_deck(tp)?;
                duel.move_sequence(tc, SEQ_DECKTOP as u8)?;
                duel.confirm_decktop(tp, 1)?;
                Ok(())
            },
            aux::stringid(ID, 3),
        )?;
    }
    Ok(())
}

fn copycon(duel: &Duel, a: &Args) -> bool {
    duel.is_existing_matching_card(
        |c, duel| c.is_summon_location(duel, LOCATION_EXTRA),
        a.tp,
        0,
        LOCATION_MZONE,
        1,
        None,
    )
}

fn copycost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    a.e.set_label(duel, 100);
    let c = a.e.get_handler(duel);
    if chk == 0 {
        return Ok(c.is_able_to_remove_as_cost(duel));
    }
    duel.remove(c, POS_FACEUP, REASON_COST)?;
    Ok(false)
}

fn copyfilter(c: CardId, duel: &Duel) -> bool {
    c.is_able_to_grave_as_cost(duel)
        && c.is_set_card(duel, &[SET_RANK_UP_MAGIC])
        && c.is_spell(duel)
        && c.check_activate_effect(duel, true, true).is_some()
}

fn copytg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if chkc.is_some() {
        // The Lua's chkc check calls an undefined `tg`.
        return Ok(false);
    }
    if chk == 0 {
        if e.get_label(duel) != 100 {
            return Ok(false);
        }
        e.set_label(duel, 0);
        return Ok(duel.is_existing_matching_card(copyfilter, tp, LOCATION_HAND, 0, 1, None));
    }
    e.set_label(duel, 0);
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TOGRAVE);
    let g = duel.select_matching_card(tp, copyfilter, tp, LOCATION_HAND, 0, 1, 1, None)?;
    duel.send_to_grave(g, REASON_COST)?;
    let Some(te) = g.get_first().and_then(|sc| sc.check_activate_effect(duel, true, true)) else {
        duel.reach_unported(concat!(
            file!(),
            ":",
            line!(),
            " (the copied Spell has no activation, where the Lua errors)"
        ));
        return Ok(false);
    };
    let label_object = te.get_label_object(duel);
    e.set_label(duel, te.get_label(duel));
    e.set_label_object(duel, label_object);
    match te.get_target(duel) {
        Target::Action(tg) => {
            tg(duel, a, 1, None)?;
        }
        other => unported!(duel, !matches!(other, Target::None)),
    }
    let label_object = e.get_label_object(duel);
    te.set_label(duel, e.get_label(duel));
    te.set_label_object(duel, label_object);
    e.set_label_object(duel, Some(te));
    duel.clear_operation_info(0);
    Ok(false)
}

fn copyop(duel: &mut Duel, a: &Args) -> Script<()> {
    let e = a.e;
    let Some(te) = e.get_label_object(duel) else { return Ok(()) };
    let label_object = te.get_label_object(duel);
    e.set_label(duel, te.get_label(duel));
    e.set_label_object(duel, label_object);
    let op = te.get_operation(duel);
    unported!(duel, !matches!(op, None | Some(Operation::Event(_) | Operation::Promptless(_))));
    if let Some(Operation::Event(op) | Operation::Promptless(op)) = op {
        op(duel, a)?;
    }
    let label_object = e.get_label_object(duel);
    te.set_label(duel, e.get_label(duel));
    te.set_label_object(duel, label_object);
    Ok(())
}
