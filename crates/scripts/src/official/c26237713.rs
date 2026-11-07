// Pendulum Treasure
use crate::prelude::*;
use crate::testing::{DECK, HAND};

const ID: u32 = 26237713;

// In the hand, with a Pendulum Normal Monster in the Deck, Flash Knight.
pub const SUPPORT: &[(u32, u16)] = &[(ID, HAND), (17390179, DECK)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_count_limit_full(duel, 1, ID, EFFECT_COUNT_CODE_OATH);
    e1.set_target(duel, target);
    e1.set_operation(duel, activate);
    c.register_effect(duel, e1);
}

fn tefilter(c: CardId, duel: &Duel) -> bool {
    c.is_type(duel, TYPE_PENDULUM) && !c.is_forbidden(duel)
}

fn target(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(tefilter, a.tp, LOCATION_DECK, 0, 1, None));
    }
    Ok(false)
}

fn activate(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    duel.hint(HINT_SELECTMSG, tp, aux::stringid(ID, 0));
    let g = duel.select_matching_card(tp, tefilter, tp, LOCATION_DECK, 0, 1, 1, None)?;
    if !g.is_empty() {
        duel.send_to_extra_p(g, tp, REASON_EFFECT)?;
    }
    Ok(())
}
