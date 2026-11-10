// That Grass Looks Greener
use crate::prelude::*;
use crate::testing::HAND;

const ID: u32 = 11110587;

pub const SUPPORT: &[(u32, u16)] = &[(ID, HAND)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_category(duel, CATEGORY_DECKDES);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_condition(duel, condition);
    e1.set_target(duel, target);
    e1.set_operation(duel, activate);
    c.register_effect(duel, e1);
}

fn deck_difference(duel: &Duel, tp: u8) -> i64 {
    duel.get_field_group_count(tp, LOCATION_DECK, 0) as i64 - duel.get_field_group_count(tp, 0, LOCATION_DECK) as i64
}

fn condition(duel: &Duel, a: &Args) -> bool {
    deck_difference(duel, a.tp) > 0
}

fn target(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (tp, ct) = (a.tp, deck_difference(duel, a.tp));
    if chk == 0 {
        return Ok(ct > 0 && duel.is_player_can_discard_deck(tp, ct as usize));
    }
    duel.set_operation_info(0, CATEGORY_DECKDES, None, 0, tp, ct as i32);
    Ok(false)
}

fn activate(duel: &mut Duel, a: &Args) -> Script<()> {
    let ct = deck_difference(duel, a.tp);
    if ct > 0 {
        duel.discard_deck(a.tp, ct as u16, REASON_EFFECT)?;
    }
    Ok(())
}
