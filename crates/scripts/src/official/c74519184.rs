// Hand Destruction
use crate::prelude::*;
use crate::testing::HAND;

const ID: u32 = 74519184;

// Copies in the hand.
pub const SUPPORT: &[(u32, u16)] = &[(ID, HAND), (ID, HAND)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_category(duel, CATEGORY_DRAW + CATEGORY_HANDES);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_target(duel, target);
    e1.set_operation(duel, activate);
    c.register_effect(duel, e1);
}

fn target(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        let mut h1 = duel.get_field_group_count(tp, LOCATION_HAND, 0);
        if a.e.get_handler(duel).is_location(duel, LOCATION_HAND) {
            h1 -= 1;
        }
        let h2 = duel.get_field_group_count(tp, 0, LOCATION_HAND);
        return Ok(h1 > 1 && h2 > 1 && duel.is_player_can_draw(tp, 2) && duel.is_player_can_draw(1 - tp, 2));
    }
    duel.set_operation_info(0, CATEGORY_HANDES, None, 0, PLAYER_ALL, 2);
    duel.set_operation_info(0, CATEGORY_DRAW, None, 0, PLAYER_ALL, 2);
    Ok(false)
}

fn activate(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    let h1 = duel.get_field_group_count(tp, LOCATION_HAND, 0);
    let h2 = duel.get_field_group_count(tp, 0, LOCATION_HAND);
    if h1 < 2 || h2 < 2 {
        return Ok(());
    }
    let turnp = duel.get_turn_player();
    duel.hint(HINT_SELECTMSG, turnp, HINTMSG_TOGRAVE);
    let mut g1 = duel.select_matching_card(turnp, aux::TRUE, turnp, LOCATION_HAND, 0, 2, 2, None)?;
    duel.confirm_cards(1 - turnp, g1)?;
    duel.hint(HINT_SELECTMSG, 1 - turnp, HINTMSG_TOGRAVE);
    let g2 = duel.select_matching_card(1 - turnp, aux::TRUE, 1 - turnp, LOCATION_HAND, 0, 2, 2, None)?;
    g1.merge(g2);
    if !g1.is_empty() && duel.send_to_grave(g1, REASON_EFFECT)? > 0 {
        let og = duel.get_operated_group();
        if !og.is_exists(duel, |c, duel| c.is_location(duel, LOCATION_GRAVE), 1, None) {
            return Ok(());
        }
        duel.break_effect()?;
        duel.draw(turnp, 2, REASON_EFFECT)?;
        duel.draw(1 - turnp, 2, REASON_EFFECT)?;
    }
    Ok(())
}
