// Gravekeeper's Descendant
use crate::prelude::*;
use crate::testing::{Check, MZONE, OPPONENT};

const ID: u32 = 30213599;

// On the field with Gravekeeper's Spy and Guard to release, and the opponent's monsters to destroy.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (24317029, MZONE),
    (24317029, MZONE),
    (37101832, MZONE),
    (26202165, MZONE | OPPONENT),
    (26202165, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_DESTROY);
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_cost(duel, cost);
    e1.set_target(duel, target);
    e1.set_operation(duel, operation);
    c.register_effect(duel, e1);
}

fn costfilter(c: CardId, duel: &Duel) -> bool {
    c.is_faceup(duel) && c.is_set_card(duel, &[SET_GRAVEKEEPERS])
}

fn cost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let dg = duel.get_matching_group(|c, duel| c.is_can_be_effect_target(duel, e), tp, 0, LOCATION_ONFIELD, None);
    let check = aux::release_check_target(dg);
    let handler = Some(e.get_handler(duel).into());
    if chk == 0 {
        return Ok(duel.check_release_group_cost(tp, costfilter, (1, 1), false, Some(&check), handler));
    }
    let sg = duel.select_release_group_cost(tp, costfilter, (1, 1), false, Some(&check), handler)?;
    duel.release(sg, REASON_COST)?;
    Ok(false)
}

fn target(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_controler(duel, 1 - tp) && chkc.is_on_field(duel));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(aux::TRUE, tp, 0, LOCATION_ONFIELD, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_DESTROY);
    let g = duel.select_target(tp, aux::TRUE, tp, 0, LOCATION_ONFIELD, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_DESTROY, g, 1, 0, 0);
    Ok(false)
}

fn operation(duel: &mut Duel, a: &Args) -> Script<()> {
    if let Some(tc) = duel.get_first_target()
        && tc.is_relate_to_effect(duel, a.e)
    {
        duel.destroy(tc, REASON_EFFECT)?;
    }
    Ok(())
}
