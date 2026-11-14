// Destiny Draw
use crate::prelude::*;
use crate::testing::HAND;

pub const LISTED_SERIES: &[u16] = &[SET_DESTINY_HERO];
// Destiny HERO - Dark Angel in the hand to discard.
pub const SUPPORT: &[(u32, u16)] = &[(26964762, HAND)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_category(duel, CATEGORY_DRAW);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_property(duel, EFFECT_FLAG_PLAYER_TARGET);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_cost(duel, cost);
    e1.set_target(duel, stock::draw_target::<2>);
    e1.set_operation(duel, stock::draw);
    c.register_effect(duel, e1);
}

fn filter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_DESTINY_HERO]) && c.is_discardable(duel)
}

fn cost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(filter, tp, LOCATION_HAND, 0, 1, None));
    }
    duel.discard_hand(tp, filter, 1, 1, REASON_COST | REASON_DISCARD)?;
    Ok(false)
}
