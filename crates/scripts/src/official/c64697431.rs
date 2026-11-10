// Hey, Trunade!
use crate::prelude::*;
use crate::testing::{HAND, OPPONENT, SZONE};

const ID: u32 = 64697431;

// In the hand, with Set Spells and Traps on the field to return.
pub const SUPPORT: &[(u32, u16)] = &[(ID, HAND), (ID, HAND), (14087893, SZONE), (14087893, SZONE | OPPONENT)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_category(duel, CATEGORY_TOHAND);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_target(duel, target);
    e1.set_operation(duel, activate);
    c.register_effect(duel, e1);
}

fn filter(c: CardId, duel: &Duel) -> bool {
    c.is_facedown(duel) && c.is_spell_trap(duel) && c.is_able_to_hand(duel)
}

fn target(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let c = a.e.get_handler(duel);
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(filter, a.tp, LOCATION_ONFIELD, LOCATION_ONFIELD, 1, Some(c.into())));
    }
    let sg = duel.get_matching_group(filter, a.tp, LOCATION_ONFIELD, LOCATION_ONFIELD, Some(c.into()));
    duel.set_operation_info(0, CATEGORY_TOHAND, Some(sg), sg.len() as u8, 0, 0);
    Ok(false)
}

fn activate(duel: &mut Duel, a: &Args) -> Script<()> {
    let c = a.e.get_handler(duel);
    let sg = duel.get_matching_group(filter, a.tp, LOCATION_ONFIELD, LOCATION_ONFIELD, Some(c.into()));
    duel.send_to_hand(sg, PLAYER_NONE, REASON_EFFECT)?;
    Ok(())
}
