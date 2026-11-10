// Regenerating Mummy
use crate::prelude::*;
use crate::testing::{Check, HAND, OPPONENT};

const ID: u32 = 70821187;

// Copies in the hand, for the opponent's Card Destruction to discard.
pub const SUPPORT: &[(u32, u16)] = &[(ID, HAND), (ID, HAND), (72892473, HAND | OPPONENT)];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_TOHAND);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_F);
    e1.set_code(duel, EVENT_TO_GRAVE);
    e1.set_condition(duel, condition);
    e1.set_target(duel, target);
    e1.set_operation(duel, operation);
    c.register_effect(duel, e1);
}

fn condition(duel: &Duel, a: &Args) -> bool {
    a.e.get_handler(duel).is_previous_location(duel, LOCATION_HAND) && a.rp != a.tp && a.r & REASON_EFFECT == REASON_EFFECT
}

fn target(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let c = a.e.get_handler(duel);
    if chk == 0 {
        return Ok(c.is_relate_to_effect(duel, a.e));
    }
    duel.set_operation_info(0, CATEGORY_TOHAND, Some(c.into()), 1, 0, 0);
    Ok(false)
}

fn operation(duel: &mut Duel, a: &Args) -> Script<()> {
    let c = a.e.get_handler(duel);
    if c.is_relate_to_effect(duel, a.e) && duel.send_to_hand(c, PLAYER_NONE, REASON_EFFECT)? == 1 {
        duel.confirm_cards(1 - a.tp, c)?;
    }
    Ok(())
}
