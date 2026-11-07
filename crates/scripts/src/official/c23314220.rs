// Spellbook of Knowledge
use crate::prelude::*;
use crate::testing::{Check, HAND, MZONE};

const ID: u32 = 23314220;

pub const LISTED_NAMES: &[u32] = &[ID];

// In the hand with other Spellbooks in the hand to send, and a Spellcaster on the field.
pub const SUPPORT: &[(u32, u16)] = &[(ID, HAND), (89739383, HAND), (14824019, MZONE)];
pub const CHECKS: &[Check] = &[Check::Resolves];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_category(duel, CATEGORY_TOGRAVE + CATEGORY_DRAW);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_count_limit_full(duel, 1, ID, EFFECT_COUNT_CODE_OATH);
    e1.set_target(duel, target);
    e1.set_operation(duel, activate);
    c.register_effect(duel, e1);
}

fn filter(c: CardId, duel: &Duel) -> bool {
    (((c.is_faceup(duel) || c.is_location(duel, LOCATION_HAND)) && c.is_set_card(duel, &[SET_SPELLBOOK]))
        || (c.is_faceup(duel) && c.is_location(duel, LOCATION_MZONE) && c.is_race(duel, RACE_SPELLCASTER)))
        && !c.is_code(duel, ID)
        && c.is_able_to_grave(duel)
}

fn target(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if chk == 0 {
        let handler = e.get_handler(duel);
        return Ok(duel.is_player_can_draw(tp, 2)
            && duel.is_existing_matching_card(filter, tp, LOCATION_HAND | LOCATION_ONFIELD, 0, 1, Some(handler.into())));
    }
    duel.set_operation_info(0, CATEGORY_TOGRAVE, None, 1, tp, (LOCATION_HAND | LOCATION_ONFIELD) as i32);
    duel.set_operation_info(0, CATEGORY_DRAW, None, 0, tp, 2);
    Ok(false)
}

fn activate(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TOGRAVE);
    let handler = a.e.get_handler(duel);
    let g = duel.select_matching_card(tp, filter, tp, LOCATION_HAND | LOCATION_ONFIELD, 0, 1, 1, Some(handler.into()))?;
    if let Some(tc) = g.get_first()
        && duel.send_to_grave(tc, REASON_EFFECT)? > 0
        && tc.is_location(duel, LOCATION_GRAVE)
    {
        duel.draw(tp, 2, REASON_EFFECT)?;
    }
    Ok(())
}
