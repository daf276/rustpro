// Nobleman of Extermination
use crate::prelude::*;
use crate::testing::{Check, DECK, HAND, OPPONENT, SZONE};

const ID: u32 = 17449108;

// Set Bottomless Trap Holes on both sides to destroy, with copies in both Decks to banish.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (29401950, SZONE),
    (29401950, SZONE | OPPONENT),
    (29401950, SZONE | OPPONENT),
    (29401950, DECK),
    (29401950, DECK | OPPONENT),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
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
    let (tp, c) = (a.tp, a.e.get_handler(duel));
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_SZONE) && filter(chkc, duel) && chkc != c);
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(filter, tp, LOCATION_SZONE, LOCATION_SZONE, 1, Some(c.into())));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_DESTROY);
    let g = duel.select_target(tp, filter, tp, LOCATION_SZONE, LOCATION_SZONE, 1, 1, Some(c.into()))?;
    duel.set_operation_info(0, CATEGORY_DESTROY, g, 1, tp, 0);
    duel.set_operation_info(0, CATEGORY_REMOVE, g, 1, tp, 0);
    duel.set_possible_operation_info(0, CATEGORY_REMOVE, None, 1, PLAYER_ALL, LOCATION_DECK as i32);
    Ok(false)
}

fn activate(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    let Some(tc) = duel.get_first_target() else { return Ok(()) };
    if tc.is_facedown(duel)
        && tc.is_relate_to_effect(duel, a.e)
        && duel.destroy_full(tc, REASON_EFFECT, LOCATION_REMOVED, duel.get_reason_player())? > 0
        && tc.is_trap(duel)
    {
        let code = tc.get_code(duel);
        let g = duel.get_field_group(tp, 0, LOCATION_DECK);
        duel.confirm_cards(tp, g)?;
        let g = duel.get_field_group(tp, LOCATION_DECK, 0);
        duel.confirm_cards(1 - tp, g)?;
        duel.shuffle_deck(tp)?;
        duel.shuffle_deck(1 - tp)?;
        let rmvg = duel.get_matching_group(|c, duel| c.is_code(duel, code), tp, LOCATION_DECK, LOCATION_DECK, None);
        if !rmvg.is_empty() {
            duel.remove(rmvg, POS_FACEUP, REASON_EFFECT)?;
        }
    }
    Ok(())
}
