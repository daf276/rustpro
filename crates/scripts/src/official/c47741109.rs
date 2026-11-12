// Sylvan Snapdrassinagon
use crate::prelude::*;
use crate::testing::{DECK, HAND, MZONE};

const ID: u32 = 47741109;

// Copies in the hand and on the field to send to the Graveyard, and Dark Plant (a Plant) in the Deck to excavate.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, MZONE),
    (ID, MZONE),
    (13193642, DECK),
    (13193642, DECK),
    (13193642, DECK),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_DECKDES);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e1.set_code(duel, EVENT_TO_GRAVE);
    e1.set_property(duel, EFFECT_FLAG_DELAY);
    e1.set_condition(duel, condition);
    e1.set_target(duel, target);
    e1.set_operation(duel, operation);
    c.register_effect(duel, e1);
    let e2 = e1.clone_effect(duel);
    e2.set_condition(duel, condition2);
    c.register_effect(duel, e2);
}

fn condition(duel: &Duel, a: &Args) -> bool {
    a.e.get_handler(duel).is_previous_location(duel, LOCATION_HAND | LOCATION_ONFIELD)
}

fn condition2(duel: &Duel, a: &Args) -> bool {
    let c = a.e.get_handler(duel);
    c.is_previous_location(duel, LOCATION_DECK) && c.is_reason(duel, REASON_EXCAVATE)
}

fn target(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    Ok(chk == 0 && duel.is_player_can_discard_deck(a.tp, 1))
}

fn operation(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    if !duel.is_player_can_discard_deck(tp, 1) {
        return Ok(());
    }
    duel.confirm_decktop(tp, 1)?;
    let g = duel.get_decktop_group(tp, 1);
    let Some(tc) = g.get_first() else { return Ok(()) };
    if tc.is_race(duel, RACE_PLANT) {
        duel.disable_shuffle_check(true);
        duel.send_to_grave(g, REASON_EFFECT | REASON_EXCAVATE)?;
    } else {
        duel.move_sequence(tc, 1)?;
    }
    Ok(())
}
