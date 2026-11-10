// Dragged Down into the Grave
use crate::prelude::*;
use crate::testing::HAND;

const ID: u32 = 16435215;

pub const SUPPORT: &[(u32, u16)] = &[(ID, HAND)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_category(duel, CATEGORY_TOGRAVE + CATEGORY_HANDES + CATEGORY_DRAW);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_condition(duel, condition);
    e1.set_target(duel, target);
    e1.set_operation(duel, activate);
    c.register_effect(duel, e1);
}

fn condition(duel: &Duel, a: &Args) -> bool {
    if duel.get_field_group_count(a.tp, 0, LOCATION_HAND) == 0 {
        return false;
    }
    let ct = usize::from(a.e.get_handler(duel).is_location(duel, LOCATION_HAND));
    duel.get_field_group_count(a.tp, LOCATION_HAND, 0) > ct
}

fn target(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(duel.is_player_can_draw(a.tp, 1) && duel.is_player_can_draw(1 - a.tp, 1));
    }
    duel.set_operation_info(0, CATEGORY_HANDES, None, 0, PLAYER_ALL, 1);
    duel.set_operation_info(0, CATEGORY_DRAW, None, 0, PLAYER_ALL, 1);
    Ok(false)
}

fn activate(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    if duel.get_field_group_count(tp, 0, LOCATION_HAND) == 0 || duel.get_field_group_count(tp, LOCATION_HAND, 0) == 0 {
        return Ok(());
    }
    let g1 = duel.get_field_group(tp, 0, LOCATION_HAND);
    let g2 = duel.get_field_group(tp, LOCATION_HAND, 0);
    duel.confirm_cards(tp, g1)?;
    duel.confirm_cards(1 - tp, g2)?;
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_DISCARD);
    let mut sg1 = g1.select(duel, tp, 1, 1, None)?;
    duel.hint(HINT_SELECTMSG, 1 - tp, HINTMSG_DISCARD);
    let sg2 = g2.select(duel, 1 - tp, 1, 1, None)?;
    sg1.merge(sg2);
    duel.send_to_grave(sg1, REASON_EFFECT | REASON_DISCARD)?;
    duel.shuffle_hand(tp)?;
    duel.shuffle_hand(1 - tp)?;
    duel.break_effect()?;
    duel.draw(tp, 1, REASON_EFFECT)?;
    duel.draw(1 - tp, 1, REASON_EFFECT)?;
    Ok(())
}
