// A Cat of Ill Omen
use crate::prelude::*;
use crate::testing::{DECK, HAND, MZONE, OPPONENT};

const ID: u32 = 24140059;

pub const LISTED_NAMES: &[u32] = &[CARD_NECROVALLEY];

// Face-down copies on the field to be flipped by the opponent's attackers, Necrovalley in hand to activate, and Traps in the Deck.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (ID, MZONE),
    (ID, MZONE),
    (ID, MZONE),
    (CARD_NECROVALLEY, HAND),
    (22359980, DECK),
    (22359980, DECK),
    (13597785, MZONE | OPPONENT),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_category(duel, CATEGORY_TOHAND + CATEGORY_SEARCH);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_FLIP);
    e1.set_operation(duel, operation);
    c.register_effect(duel, e1);
}

fn operation(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    duel.hint(HINT_SELECTMSG, tp, aux::stringid(ID, 1));
    let g = duel.select_matching_card(tp, CardId::is_trap, tp, LOCATION_DECK, 0, 1, 1, None)?;
    let Some(tc) = g.get_first() else { return Ok(()) };
    if duel.is_environment(CARD_NECROVALLEY) && tc.is_able_to_hand(duel) && duel.select_yes_no(tp, aux::stringid(ID, 0))? {
        duel.send_to_hand(tc, PLAYER_NONE, REASON_EFFECT)?;
        duel.confirm_cards(1 - tp, tc)?;
    } else {
        duel.shuffle_deck(tp)?;
        duel.move_sequence(tc, 0)?;
        duel.confirm_decktop(tp, 1)?;
    }
    Ok(())
}
