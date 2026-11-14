// Snake Rain
use crate::prelude::*;
use crate::testing::{DECK, HAND};

const ID: u32 = 17189677;

// Reptile Normal Monsters (Gigobyte, Grappler, Armored Lizard, Venom Cobra, Worm Drake) to send.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (53776525, DECK),
    (2906250, DECK),
    (15480588, DECK),
    (22499463, DECK),
    (73216412, DECK),
    (53776525, DECK),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_category(duel, CATEGORY_TOGRAVE);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_cost(duel, cost);
    e1.set_target(duel, target);
    e1.set_operation(duel, activate);
    c.register_effect(duel, e1);
}

fn cost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        let c = a.e.get_handler(duel);
        return Ok(duel.is_existing_matching_card(CardId::is_discardable, tp, LOCATION_HAND, 0, 1, Some(c.into())));
    }
    duel.discard_hand(tp, CardId::is_discardable, 1, 1, REASON_COST | REASON_DISCARD)?;
    Ok(false)
}

fn tgfilter(c: CardId, duel: &Duel) -> bool {
    c.is_race(duel, RACE_REPTILE) && c.is_monster(duel) && c.is_able_to_grave(duel)
}

fn target(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(tgfilter, a.tp, LOCATION_DECK, 0, 4, None));
    }
    duel.set_operation_info(0, CATEGORY_TOGRAVE, None, 4, a.tp, LOCATION_DECK as i32);
    Ok(false)
}

fn activate(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    let g = duel.get_matching_group(tgfilter, tp, LOCATION_DECK, 0, None);
    if g.len() >= 4 {
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_TOGRAVE);
        let sg = g.select(duel, tp, 4, 4, None)?;
        duel.send_to_grave(sg, REASON_EFFECT)?;
    }
    Ok(())
}
