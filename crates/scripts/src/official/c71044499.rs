// Nobleman of Crossout
use crate::prelude::*;
use crate::testing::{DECK, HAND, MZONE, OPPONENT};

const ID: u32 = 71044499;

// In the hand, with Flip monsters on both sides to banish, and their copies in both Decks.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (32362575, MZONE),
    (32362575, MZONE | OPPONENT),
    (32362575, DECK),
    (32362575, DECK | OPPONENT),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_category(duel, CATEGORY_DESTROY + CATEGORY_REMOVE);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_target(duel, target);
    e1.set_operation(duel, activate);
    c.register_effect(duel, e1);
}

fn filter(c: CardId, duel: &Duel) -> bool {
    c.is_facedown(duel) && c.is_able_to_remove(duel)
}

fn target(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_MZONE) && filter(chkc, duel));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(filter, tp, LOCATION_MZONE, LOCATION_MZONE, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_DESTROY);
    let g = duel.select_target(tp, filter, tp, LOCATION_MZONE, LOCATION_MZONE, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_DESTROY, g, 1, 0, 0);
    duel.set_operation_info(0, CATEGORY_REMOVE, g, 1, 0, 0);
    Ok(false)
}

fn activate(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    let Some(tc) = duel.get_first_target() else { return Ok(()) };
    if tc.is_facedown(duel)
        && tc.is_relate_to_effect(duel, a.e)
        && duel.destroy_full(tc, REASON_EFFECT, LOCATION_REMOVED, duel.get_reason_player())? != 0
        && tc.is_type(duel, TYPE_FLIP)
    {
        let code = tc.get_code(duel);
        let g = duel.get_matching_group(|c, duel| c.is_code(duel, code), tp, LOCATION_DECK, LOCATION_DECK, None);
        duel.remove(g, POS_FACEUP, REASON_EFFECT)?;
        let g = duel.get_field_group(tp, 0, LOCATION_DECK);
        duel.confirm_cards(tp, g)?;
        let g = duel.get_field_group(tp, LOCATION_DECK, 0);
        duel.confirm_cards(1 - tp, g)?;
        duel.shuffle_deck(tp)?;
        duel.shuffle_deck(1 - tp)?;
    }
    Ok(())
}
