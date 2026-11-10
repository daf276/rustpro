// Card Destruction
use crate::prelude::*;

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_category(duel, CATEGORY_HANDES + CATEGORY_DRAW);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_target(duel, target);
    e1.set_operation(duel, activate);
    c.register_effect(duel, e1);
}

fn target(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        let in_hand = a.e.get_handler(duel).is_location(duel, LOCATION_HAND);
        let h1 = duel.get_field_group_count(tp, LOCATION_HAND, 0) - usize::from(in_hand);
        let h2 = duel.get_field_group_count(tp, 0, LOCATION_HAND);
        return Ok(h1 + h2 > 0 && (duel.is_player_can_draw(tp, h1) || h1 == 0) && (duel.is_player_can_draw(1 - tp, 0) || h2 == 0));
    }
    duel.set_operation_info(0, CATEGORY_HANDES, None, 0, PLAYER_ALL, 1);
    duel.set_operation_info(0, CATEGORY_DRAW, None, 0, PLAYER_ALL, 1);
    Ok(false)
}

fn activate(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    let h1 = duel.get_field_group_count(tp, LOCATION_HAND, 0) as u32;
    let h2 = duel.get_field_group_count(tp, 0, LOCATION_HAND) as u32;
    let g = duel.get_field_group(tp, LOCATION_HAND, LOCATION_HAND);
    duel.send_to_grave(g, REASON_EFFECT | REASON_DISCARD)?;
    duel.break_effect()?;
    duel.draw(tp, h1, REASON_EFFECT)?;
    duel.draw(1 - tp, h2, REASON_EFFECT)?;
    Ok(())
}
