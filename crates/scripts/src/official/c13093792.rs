// Destiny HERO - Diamond Dude
use engine::{Operation, Target};

use crate::prelude::*;
use crate::testing::{DECK, MZONE};

const ID: u32 = 13093792;

// On the field, with Normal Spells (Polymerization, Monster Reborn, Pot of Greed, Dark Hole, Raigeki) in the Deck to excavate.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (CARD_POLYMERIZATION, DECK),
    (83764718, DECK),
    (55144522, DECK),
    (53129443, DECK),
    (12580477, DECK),
    (12580477, DECK),
    (12580477, DECK),
    (12580477, DECK),
    (12580477, DECK),
    (12580477, DECK),
    (12580477, DECK),
    (12580477, DECK),
    (12580477, DECK),
    (12580477, DECK),
    (12580477, DECK),
    (12580477, DECK),
    (12580477, DECK),
    (12580477, DECK),
    (12580477, DECK),
    (12580477, DECK),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_DECKDES);
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_count_limit(duel, 1, 0);
    e1.set_target(duel, target);
    e1.set_operation(duel, operation);
    c.register_effect(duel, e1);
}

fn target(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.get_field_group_count(tp, LOCATION_DECK, 0) > 0);
    }
    duel.set_possible_operation_info(0, CATEGORY_TOGRAVE, None, 1, tp, LOCATION_DECK as i32);
    Ok(false)
}

fn operation(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    if duel.get_field_group_count(tp, LOCATION_DECK, 0) == 0 {
        return Ok(());
    }
    duel.confirm_decktop(tp, 1)?;
    let g = duel.get_decktop_group(tp, 1);
    let Some(tc) = g.get_first() else { return Ok(()) };
    if !tc.is_normal_spell(duel) {
        return duel.move_to_deck_bottom(tc.into());
    }
    duel.disable_shuffle_check(true);
    duel.send_to_grave(g, REASON_EFFECT)?;
    if let Some(ae) = tc.get_activate_effect(duel)
        && tc.is_location(duel, LOCATION_GRAVE)
    {
        let e1 = duel.create_effect(tc);
        e1.set_description(duel, ae.get_description(duel));
        e1.set_type(duel, EFFECT_TYPE_IGNITION);
        e1.set_count_limit(duel, 1, 0);
        e1.set_range(duel, LOCATION_GRAVE);
        let reset = RESET_EVENT | RESETS_STANDARD | RESET_CONTROL | RESET_PHASE | PHASE_END | (RESET_SELF_TURN & !RESET_TOFIELD);
        e1.set_reset_full(duel, reset, 2);
        e1.set_condition(duel, spellcon);
        e1.set_target(duel, spelltg);
        e1.set_operation(duel, spellop);
        tc.register_effect(duel, e1);
    }
    Ok(())
}

fn spellcon(duel: &Duel, a: &Args) -> bool {
    a.e.get_handler(duel).get_turn_id(duel) != duel.get_turn_count()
}

fn spelltg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let e = a.e;
    let Some(ae) = e.get_handler(duel).get_activate_effect(duel) else {
        return Ok(false);
    };
    let ftg = match ae.get_target(duel) {
        Target::Action(ftg) => Some(ftg),
        other => {
            unported!(duel, !matches!(other, Target::None));
            None
        }
    };
    if chk == 0 {
        return ftg.map_or(Ok(true), |ftg| ftg(duel, a, chk, None));
    }
    e.set_property(
        duel,
        if ae.is_has_property(duel, EFFECT_FLAG_CARD_TARGET) {
            EFFECT_FLAG_CARD_TARGET
        } else {
            0
        },
    );
    if let Some(ftg) = ftg {
        ftg(duel, a, chk, None)?;
    }
    Ok(false)
}

fn spellop(duel: &mut Duel, a: &Args) -> Script<()> {
    let Some(ae) = a.e.get_handler(duel).get_activate_effect(duel) else {
        return Ok(());
    };
    let op = ae.get_operation(duel);
    unported!(duel, !matches!(op, Some(Operation::Event(_) | Operation::Promptless(_))));
    match op {
        Some(Operation::Event(fop) | Operation::Promptless(fop)) => fop(duel, a),
        _ => Ok(()),
    }
}
