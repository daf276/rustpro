// Raise Moon Lamp Croucier
use super::c100458016::{register_draw_summon, track_searches};
use super::c100458019::{register_discard_draw, register_summoned_this_way};
use crate::prelude::*;
use crate::testing::{Check, DECK, HAND, OPPONENT, SZONE};

const ID: u32 = 100458020;

// Copies to draw and to discard, and the opponent's Set Mirror Force to place on the bottom of the Deck.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, DECK),
    (ID, DECK),
    (ID, DECK),
    (ID, HAND),
    (44095762, SZONE | OPPONENT),
    (44095762, SZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
    Check::ResolvesEffect(aux::stringid(ID, 2)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    register_draw_summon::<1>(duel, c, ID);
    register_summoned_this_way(duel, c, ID, tdtg);
    register_discard_draw::<ID>(duel, c);
    track_searches::<ID, { RESET_PHASE | PHASE_END }>(duel, c);
}

fn tdfilter(c: CardId, duel: &Duel) -> bool {
    c.is_spell_trap(duel) && c.is_able_to_deck(duel)
}

fn tdtg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_controler(duel, 1 - tp) && chkc.is_on_field(duel) && tdfilter(chkc, duel));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(tdfilter, tp, 0, LOCATION_ONFIELD, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TODECK);
    let g = duel.select_target(tp, tdfilter, tp, 0, LOCATION_ONFIELD, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_TODECK, g, 1, tp, SEQ_DECKBOTTOM);
    Ok(false)
}
