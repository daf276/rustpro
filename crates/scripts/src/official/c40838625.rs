// Quaking Mirror Force
use crate::prelude::*;
use crate::testing::{MZONE, OPPONENT, SZONE};

const ID: u32 = 40838625;

// Set on the field against attacking Sangan.
pub const SUPPORT: &[(u32, u16)] = &[(ID, SZONE), (ID, SZONE), (26202165, MZONE | OPPONENT), (26202165, MZONE | OPPONENT)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_category(duel, CATEGORY_POSITION + CATEGORY_SET);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_ATTACK_ANNOUNCE);
    e1.set_condition(duel, |duel, a| duel.is_turn_player(1 - a.tp));
    e1.set_target(duel, target);
    e1.set_operation(duel, activate);
    c.register_effect(duel, e1);
}

fn filter(c: CardId, duel: &Duel) -> bool {
    c.is_attack_pos(duel) && c.is_can_turn_set(duel)
}

fn target(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(filter, tp, 0, LOCATION_MZONE, 1, None));
    }
    let g = duel.get_matching_group(filter, tp, 0, LOCATION_MZONE, None);
    duel.set_operation_info(0, CATEGORY_POSITION, Some(g), g.len() as u8, tp, POS_FACEDOWN_DEFENSE as i32);
    Ok(false)
}

fn activate(duel: &mut Duel, a: &Args) -> Script<()> {
    let g = duel.get_matching_group(filter, a.tp, 0, LOCATION_MZONE, None);
    if duel.change_position(g, POS_FACEDOWN_DEFENSE)? != 0 {
        for tc in duel.get_operated_group() {
            let e1 = duel.create_effect(a.e.get_handler(duel));
            e1.set_description(duel, 3313);
            e1.set_property(duel, EFFECT_FLAG_CLIENT_HINT);
            e1.set_type(duel, EFFECT_TYPE_SINGLE);
            e1.set_code(duel, EFFECT_CANNOT_CHANGE_POSITION);
            e1.set_reset(duel, RESET_EVENT | RESETS_STANDARD);
            tc.register_effect(duel, e1);
        }
    }
    Ok(())
}
