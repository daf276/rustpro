// Snipe Hunter
use engine::constants::MSG_TOSS_DICE;

use crate::prelude::*;
use crate::testing::{Check, HAND};

const ID: u32 = 84290642;

pub const ROLL_DICE: bool = true;
// A second copy in the hand, to discard.
pub const SUPPORT: &[(u32, u16)] = &[(ID, HAND)];
pub const CHECKS: &[Check] = &[Check::Resolves, Check::Message(MSG_TOSS_DICE)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_DESTROY + CATEGORY_DICE);
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_cost(duel, cost);
    e1.set_target(duel, target);
    e1.set_operation(duel, activate);
    c.register_effect(duel, e1);
}

fn cost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(CardId::is_discardable, a.tp, LOCATION_HAND, 0, 1, None));
    }
    duel.discard_hand(a.tp, CardId::is_discardable, 1, 1, REASON_COST | REASON_DISCARD)?;
    Ok(false)
}

fn target(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    if let Some(chkc) = chkc {
        return Ok(chkc.is_on_field(duel));
    }
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.is_existing_target(aux::TRUE, tp, LOCATION_ONFIELD, LOCATION_ONFIELD, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_DESTROY);
    duel.select_target(tp, aux::TRUE, tp, LOCATION_ONFIELD, LOCATION_ONFIELD, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_DICE, None, 0, tp, 1);
    Ok(false)
}

fn activate(duel: &mut Duel, a: &Args) -> Script<()> {
    let Some(tc) = duel.get_first_target().filter(|tc| tc.is_relate_to_effect(duel, a.e)) else {
        return Ok(());
    };
    let d = duel.toss_dice(a.tp, 1)?.first().copied();
    if d != Some(1) && d != Some(6) {
        duel.destroy(tc, REASON_EFFECT)?;
    }
    Ok(())
}
