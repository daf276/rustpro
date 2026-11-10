// Senju of the Thousand Hands
use crate::prelude::*;
use crate::testing::{DECK, HAND, MZONE};

const ID: u32 = 23401839;

// Copies in the hand and face-down on the field to Summon, with Ritual Monsters (Megalith Och, Cyber Angel Benten) in the Deck.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, HAND),
    (ID, MZONE),
    (ID, MZONE),
    (36849933, DECK),
    (77235086, DECK),
    (77235086, DECK),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_TOHAND + CATEGORY_SEARCH);
    e1.set_type(duel, EFFECT_TYPE_TRIGGER_O + EFFECT_TYPE_SINGLE);
    e1.set_code(duel, EVENT_SUMMON_SUCCESS);
    e1.set_target(duel, tg);
    e1.set_operation(duel, op);
    c.register_effect(duel, e1);
    let e2 = e1.clone_effect(duel);
    e2.set_code(duel, EVENT_FLIP_SUMMON_SUCCESS);
    c.register_effect(duel, e2);
}

fn filter(c: CardId, duel: &Duel) -> bool {
    c.is_ritual_monster(duel) && c.is_able_to_hand(duel)
}

fn tg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(filter, a.tp, LOCATION_DECK, 0, 1, None));
    }
    duel.set_operation_info(0, CATEGORY_TOHAND, None, 1, a.tp, LOCATION_DECK as i32);
    Ok(false)
}

fn op(duel: &mut Duel, a: &Args) -> Script<()> {
    duel.hint(HINT_SELECTMSG, a.tp, HINTMSG_ATOHAND);
    let g = duel.select_matching_card(a.tp, filter, a.tp, LOCATION_DECK, 0, 1, 1, None)?;
    if !g.is_empty() {
        duel.send_to_hand(g, PLAYER_NONE, REASON_EFFECT)?;
        duel.confirm_cards(1 - a.tp, g)?;
    }
    Ok(())
}
