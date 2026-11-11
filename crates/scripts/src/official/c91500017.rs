// Toon Bookmark
use engine::constants::MSG_SELECT_EFFECTYN;

use crate::prelude::*;
use crate::testing::{Check, DECK, GRAVE, HAND, OPPONENT, SZONE};

const ID: u32 = 91500017;

pub const LISTED_NAMES: &[u32] = &[CARD_TOON_WORLD, ID];

// In the hand and the Graveyard, with Toon World on the field for the opponent's Mystical Space Typhoon to hit, and the
// Toon cards that list it in the Deck.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, GRAVE),
    (ID, GRAVE),
    (CARD_TOON_WORLD, SZONE),
    (CARD_TOON_WORLD, SZONE),
    (CARD_TOON_WORLD, DECK),
    (53094821, DECK),
    (65458948, DECK),
    (5318639, HAND | OPPONENT),
    (5318639, HAND | OPPONENT),
];
// The destruction replacement is no chain link; its prompt is the MSG_SELECT_EFFECTYN, counted for any card.
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0)), Check::Message(MSG_SELECT_EFFECTYN)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_TOHAND + CATEGORY_SEARCH);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_count_limit_full(duel, 1, ID, EFFECT_COUNT_CODE_OATH);
    e1.set_target(duel, target);
    e1.set_operation(duel, activate);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_CONTINUOUS);
    e2.set_code(duel, EFFECT_DESTROY_REPLACE);
    e2.set_range(duel, LOCATION_GRAVE);
    e2.set_target(duel, reptg);
    e2.set_value(
        duel,
        Value::Card(|duel, e, c| i32::from(repfilter(c, duel, e.get_handler_player(duel)))),
    );
    e2.set_operation(duel, repop);
    c.register_effect(duel, e2);
}

fn thfilter(c: CardId, duel: &Duel) -> bool {
    (c.lists_code(duel, CARD_TOON_WORLD) || c.is_code(duel, CARD_TOON_WORLD)) && !c.is_code(duel, ID) && c.is_able_to_hand(duel)
}

fn target(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    stock::exists_target(duel, a, chk, thfilter, LOCATION_DECK, CATEGORY_TOHAND)
}

fn activate(duel: &mut Duel, a: &Args) -> Script<()> {
    stock::search(duel, a, thfilter, LOCATION_DECK)
}

fn repfilter(c: CardId, duel: &Duel, tp: u8) -> bool {
    c.is_faceup(duel)
        && c.is_controler(duel, tp)
        && c.is_on_field(duel)
        && c.is_code(duel, CARD_TOON_WORLD)
        && !c.is_reason(duel, REASON_REPLACE)
        && c.is_reason(duel, REASON_EFFECT)
}

fn reptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (c, tp) = (a.e.get_handler(duel), a.tp);
    if chk == 0 {
        return Ok(c.is_able_to_remove(duel)
            && a.eg
                .is_some_and(|eg| eg.is_exists(duel, |tc, duel| repfilter(tc, duel, tp), 1, None)));
    }
    duel.select_effect_yes_no_full(tp, c, 96)
}

fn repop(duel: &mut Duel, a: &Args) -> Script<()> {
    let c = a.e.get_handler(duel);
    duel.remove(c, POS_FACEUP, REASON_EFFECT)?;
    Ok(())
}
