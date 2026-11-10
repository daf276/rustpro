// Creature Swap
use crate::prelude::*;
use crate::testing::{HAND, MZONE, OPPONENT};

const ID: u32 = 31036355;

// In the hand, with a Sangan on each side to swap.
pub const SUPPORT: &[(u32, u16)] = &[(ID, HAND), (26202165, MZONE), (26202165, MZONE | OPPONENT)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_category(duel, CATEGORY_CONTROL);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_target(duel, target);
    e1.set_operation(duel, activate);
    c.register_effect(duel, e1);
}

fn filter(c: CardId, duel: &Duel) -> bool {
    c.is_able_to_change_controler(duel) && (c.get_sequence(duel) < 5 || duel.get_location_count(c.get_controler(duel), LOCATION_MZONE) > 0)
}

fn both_sides(duel: &Duel, tp: u8) -> bool {
    duel.is_existing_matching_card(filter, tp, LOCATION_MZONE, 0, 1, None)
        && duel.is_existing_matching_card(filter, tp, 0, LOCATION_MZONE, 1, None)
}

fn target(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(both_sides(duel, a.tp));
    }
    duel.set_operation_info(0, CATEGORY_CONTROL, None, 0, 0, 0);
    Ok(false)
}

fn activate(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    let c = a.e.get_handler(duel);
    if !both_sides(duel, tp) {
        return Ok(());
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_CONTROL);
    let g1 = duel.select_matching_card(tp, filter, tp, LOCATION_MZONE, 0, 1, 1, None)?;
    duel.hint_selection(g1);
    duel.hint(HINT_SELECTMSG, 1 - tp, HINTMSG_CONTROL);
    let g2 = duel.select_matching_card(1 - tp, filter, 1 - tp, LOCATION_MZONE, 0, 1, 1, None)?;
    duel.hint_selection(g2);
    let (Some(c1), Some(c2)) = (g1.get_first(), g2.get_first()) else {
        return Ok(());
    };
    if duel.swap_control(c1, c2, 0, 0)? {
        let e1 = duel.create_effect(c);
        e1.set_description(duel, 3313);
        e1.set_property(duel, EFFECT_FLAG_CLIENT_HINT);
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_code(duel, EFFECT_CANNOT_CHANGE_POSITION);
        e1.set_reset(duel, RESET_PHASE | PHASE_END);
        c1.register_effect(duel, e1);
        let e2 = e1.clone_effect(duel);
        c2.register_effect(duel, e2);
    }
    Ok(())
}
