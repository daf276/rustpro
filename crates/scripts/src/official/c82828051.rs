// Earthquake
use crate::prelude::*;
use crate::testing::{MZONE, OPPONENT};

// Monsters of both players to flip into Defense Position.
pub const SUPPORT: &[(u32, u16)] = &[(CARD_DARK_MAGICIAN, MZONE), (CARD_DARK_MAGICIAN, MZONE | OPPONENT)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_category(duel, CATEGORY_POSITION);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_target(duel, target);
    e1.set_operation(duel, activate);
    c.register_effect(duel, e1);
}

fn filter(c: CardId, duel: &Duel) -> bool {
    c.is_position(duel, POS_FACEUP_ATTACK) && c.is_can_change_position(duel)
}

fn target(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(filter, tp, LOCATION_MZONE, LOCATION_MZONE, 1, None));
    }
    let g = duel.get_matching_group(filter, tp, LOCATION_MZONE, LOCATION_MZONE, None);
    duel.set_operation_info(0, CATEGORY_POSITION, Some(g), g.len() as u8, 0, 0);
    Ok(false)
}

fn activate(duel: &mut Duel, a: &Args) -> Script<()> {
    let g = duel.get_matching_group(filter, a.tp, LOCATION_MZONE, LOCATION_MZONE, None);
    duel.change_position(g, POS_FACEUP_DEFENSE)?;
    Ok(())
}
