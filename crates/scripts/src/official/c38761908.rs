// Paleozoic Dinomischus
use super::c64765016::paleozoic_spsummon;
use crate::prelude::*;
use crate::testing::{Check, GRAVE, HAND, MZONE, OPPONENT, SZONE};

const ID: u32 = 38761908;

// A copy in the Graveyard to Special Summon in response to another Set copy activating, a card in the hand to discard, and an
// opponent's monster to banish.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, GRAVE),
    (ID, SZONE),
    (ID, SZONE),
    (ID, HAND),
    (30655537, MZONE | OPPONENT),
    (30655537, HAND),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(0), Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_category(duel, CATEGORY_HANDES + CATEGORY_REMOVE);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_target(duel, target);
    e1.set_operation(duel, activate);
    e1.set_hint_timing(duel, 0, TIMINGS_CHECK_MONSTER_E);
    c.register_effect(duel, e1);
    let e2 = paleozoic_spsummon::<ID>(duel, c);
    e2.set_description(duel, aux::stringid(ID, 0));
    c.register_effect(duel, e2);
}

fn filter1(c: CardId, duel: &Duel) -> bool {
    c.is_discardable_full(duel, REASON_EFFECT, None)
}

fn filter2(c: CardId, duel: &Duel) -> bool {
    c.is_faceup(duel) && c.is_able_to_remove(duel)
}

fn target(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (tp, handler) = (a.tp, a.e.get_handler(duel));
    if let Some(chkc) = chkc {
        return Ok(chkc.is_on_field(duel) && filter2(chkc, duel) && chkc != handler);
    }
    if chk == 0 {
        return Ok(
            duel.is_existing_matching_card(filter1, tp, LOCATION_HAND, 0, 1, Some(handler.into()))
                && duel.is_existing_target(filter2, tp, LOCATION_ONFIELD, LOCATION_ONFIELD, 1, Some(handler.into())),
        );
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_REMOVE);
    let g = duel.select_target(tp, filter2, tp, LOCATION_ONFIELD, LOCATION_ONFIELD, 1, 1, Some(handler.into()))?;
    duel.set_operation_info(0, CATEGORY_REMOVE, g, 1, 0, 0);
    duel.set_operation_info(0, CATEGORY_HANDES, None, 0, tp, 1);
    Ok(false)
}

fn activate(duel: &mut Duel, a: &Args) -> Script<()> {
    let tc = duel.get_first_target();
    if duel.discard_hand(a.tp, filter1, 1, 1, REASON_EFFECT | REASON_DISCARD)? > 0
        && let Some(tc) = tc
        && tc.is_relate_to_effect(duel, a.e)
    {
        duel.remove(tc, POS_FACEUP, REASON_EFFECT)?;
    }
    Ok(())
}
