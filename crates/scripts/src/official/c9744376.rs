// Good Goblin Housekeeping
use crate::prelude::*;
use crate::testing::{GRAVE, SZONE};

const ID: u32 = 9744376;

// Set with copies in the Graveyard to count.
pub const SUPPORT: &[(u32, u16)] = &[(ID, SZONE), (ID, GRAVE), (ID, GRAVE)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_category(duel, CATEGORY_DRAW + CATEGORY_TODECK);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_property(duel, EFFECT_FLAG_PLAYER_TARGET);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_target(duel, target);
    e1.set_operation(duel, activate);
    c.register_effect(duel, e1);
}

fn target(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.is_player_can_draw(tp, 1));
    }
    duel.set_target_player(tp);
    let ct = duel.get_matching_group_count(|c, duel| c.is_code(duel, ID), tp, LOCATION_GRAVE, 0, None);
    duel.set_operation_info(0, CATEGORY_DRAW, None, 0, tp, ct as i32 + 1);
    duel.set_operation_info(0, CATEGORY_TODECK, None, 1, tp, LOCATION_HAND as i32);
    Ok(false)
}

fn activate(duel: &mut Duel, a: &Args) -> Script<()> {
    let p: u8 = duel.get_chain_info(0, CHAININFO_TARGET_PLAYER);
    let d = duel.get_matching_group_count(|c, duel| c.is_code(duel, ID), p, LOCATION_GRAVE, 0, None) as u32 + 1;
    duel.draw(p, d, REASON_EFFECT)?;
    duel.break_effect()?;
    duel.hint(HINT_SELECTMSG, a.tp, HINTMSG_TODECK);
    let g = duel.select_matching_card(p, aux::TRUE, p, LOCATION_HAND, 0, 1, 1, None)?;
    duel.send_to_deck(g, PLAYER_NONE, SEQ_DECKBOTTOM, REASON_EFFECT)?;
    Ok(())
}
