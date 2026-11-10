// Dragon Shrine
use crate::prelude::*;
use crate::testing::{DECK, HAND};

const ID: u32 = 41620959;

// In the hand, with Luster Dragon (a Normal Dragon) in the Deck to send.
pub const SUPPORT: &[(u32, u16)] = &[(ID, HAND), (11091375, DECK), (11091375, DECK), (11091375, DECK)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_category(duel, CATEGORY_TOGRAVE);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_count_limit_full(duel, 1, ID, EFFECT_COUNT_CODE_OATH);
    e1.set_target(duel, target);
    e1.set_operation(duel, activate);
    c.register_effect(duel, e1);
}

fn tgfilter(c: CardId, duel: &Duel) -> bool {
    c.is_race(duel, RACE_DRAGON) && c.is_able_to_grave(duel)
}

fn target(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(tgfilter, tp, LOCATION_DECK, 0, 1, None));
    }
    duel.set_operation_info(0, CATEGORY_TOGRAVE, None, 1, tp, LOCATION_DECK as i32);
    Ok(false)
}

fn activate(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TOGRAVE);
    let g = duel.select_matching_card(tp, tgfilter, tp, LOCATION_DECK, 0, 1, 1, None)?;
    if let Some(tc) = g.get_first()
        && duel.send_to_grave(tc, REASON_EFFECT)? > 0
        && tc.is_location(duel, LOCATION_GRAVE)
        && tc.is_race(duel, RACE_DRAGON)
        && tc.is_type(duel, TYPE_NORMAL)
        && duel.is_existing_matching_card(tgfilter, tp, LOCATION_DECK, 0, 1, None)
        && duel.select_yes_no(tp, aux::stringid(ID, 0))?
    {
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_TOGRAVE);
        let g1 = duel.select_matching_card(tp, tgfilter, tp, LOCATION_DECK, 0, 1, 1, None)?;
        duel.send_to_grave(g1, REASON_EFFECT)?;
    }
    Ok(())
}
