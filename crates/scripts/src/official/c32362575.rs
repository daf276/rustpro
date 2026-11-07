// Magical Merchant
use crate::prelude::*;
use crate::testing::{Check, DECK, MZONE};

const ID: u32 = 32362575;

// Face-down on the field to be flipped by an attack, with Spells in the Deck to excavate.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (CARD_POLYMERIZATION, DECK),
    (83764718, DECK),
    (12580477, DECK),
    (12580477, DECK),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_TOHAND + CATEGORY_SEARCH + CATEGORY_DECKDES);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_FLIP);
    e1.set_operation(duel, operation);
    c.register_effect(duel, e1);
}

fn operation(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    let g = duel.get_matching_group(CardId::is_spell_trap, tp, LOCATION_DECK, 0, None);
    let dcount = duel.get_field_group_count(tp, LOCATION_DECK, 0);
    if dcount == 0 {
        return Ok(());
    }
    let Some(spcard) = g.into_iter().max_by_key(|c| c.get_sequence(duel)) else {
        duel.confirm_decktop(tp, dcount)?;
        return duel.shuffle_deck(tp);
    };
    let seq = spcard.get_sequence(duel) as usize;
    duel.confirm_decktop(tp, dcount - seq)?;
    if spcard.is_able_to_hand(duel) {
        duel.disable_shuffle_check(true);
        duel.send_to_hand(spcard, PLAYER_NONE, REASON_EFFECT)?;
        duel.discard_deck(tp, (dcount - seq - 1) as u16, REASON_EFFECT | REASON_EXCAVATE)?;
        duel.confirm_cards(1 - tp, spcard)?;
        duel.shuffle_hand(tp)?;
    } else {
        duel.discard_deck(tp, (dcount - seq) as u16, REASON_EFFECT | REASON_EXCAVATE)?;
    }
    Ok(())
}
