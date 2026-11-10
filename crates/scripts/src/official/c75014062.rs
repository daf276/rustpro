// Spell Power Grasp
use crate::prelude::*;
use crate::testing::{DECK, HAND, MZONE};

const ID: u32 = 75014062;

pub const LISTED_NAMES: &[u32] = &[ID];
// Skilled White Magician to place a Spell Counter on and a second copy in the Deck to add.
pub const SUPPORT: &[(u32, u16)] = &[(ID, HAND), (46363422, MZONE), (ID, DECK), (ID, DECK)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_category(duel, CATEGORY_COUNTER + CATEGORY_SEARCH + CATEGORY_TOHAND);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_count_limit_full(duel, 1, ID, EFFECT_COUNT_CODE_OATH);
    e1.set_target(duel, target);
    e1.set_operation(duel, activate);
    c.register_effect(duel, e1);
}

fn filter(c: CardId, duel: &Duel) -> bool {
    c.is_faceup(duel) && c.is_can_add_counter(duel, COUNTER_SPELL, 1)
}

fn target(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_on_field(duel) && filter(chkc, duel));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(filter, tp, LOCATION_ONFIELD, LOCATION_ONFIELD, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, aux::stringid(ID, 1));
    duel.select_target(tp, filter, tp, LOCATION_ONFIELD, LOCATION_ONFIELD, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_COUNTER, None, 1, 0, i32::from(COUNTER_SPELL));
    duel.set_possible_operation_info(0, CATEGORY_TOHAND, None, 1, tp, LOCATION_DECK as i32);
    Ok(false)
}

fn tfilter(c: CardId, duel: &Duel) -> bool {
    c.is_code(duel, ID) && c.is_able_to_hand(duel)
}

fn activate(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    if let Some(tc) = duel.get_first_target()
        && tc.is_faceup(duel)
        && tc.is_relate_to_effect(duel, a.e)
        && tc.add_counter(duel, COUNTER_SPELL, 1)
        && let Some(th) = duel.get_first_matching_card(tfilter, tp, LOCATION_DECK, 0, None)
        && duel.select_yes_no(tp, aux::stringid(ID, 0))?
    {
        duel.send_to_hand(th, PLAYER_NONE, REASON_EFFECT)?;
        duel.confirm_cards(1 - tp, th)?;
    }
    Ok(())
}
