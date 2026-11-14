// Stake your Soul!
use crate::prelude::*;
use crate::testing::{Check, DECK, HAND};

const ID: u32 = 54562327;

pub const LISTED_SERIES: &[u16] = &[SET_VANQUISH_SOUL];

struct ReturnToHand;

impl aux::DelayedOperation for ReturnToHand {
    fn operate(duel: &mut Duel, ag: CardSet, _: &Args) -> Script<()> {
        duel.send_to_hand(ag, PLAYER_NONE, REASON_EFFECT)?;
        Ok(())
    }
}

// In hand with Vanquish Soul Dr. Mad Love and Heavy Borger (DARK) in the hand to reveal and in the Deck to Summon.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, HAND),
    (29280200, HAND),
    (92895501, HAND),
    (29280200, DECK),
    (29280200, DECK),
    (92895501, DECK),
    (92895501, DECK),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_count_limit_full(duel, 1, ID, EFFECT_COUNT_CODE_OATH);
    e1.set_cost(duel, cost);
    e1.set_target(duel, target);
    e1.set_operation(duel, activate);
    c.register_effect(duel, e1);
}

fn cost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    a.e.set_label(duel, -1);
    Ok(chk == 0)
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8, attr: u32, code: u32) -> bool {
    c.is_set_card(duel, &[SET_VANQUISH_SOUL])
        && c.is_attribute(duel, attr)
        && !c.is_code(duel, code)
        && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
}

fn cfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_monster(duel) && !c.is_public(duel) && {
        let (attr, code) = (c.get_attribute(duel), c.get_code(duel));
        duel.is_existing_matching_card(|c, duel| spfilter(c, duel, e, tp, attr, code), tp, LOCATION_DECK, 0, 1, None)
    }
}

fn target(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if chk == 0 {
        let res = e.get_label(duel) == -1
            && duel.get_location_count(tp, LOCATION_MZONE) > 0
            && duel.is_existing_matching_card(|c, duel| cfilter(c, duel, e, tp), tp, LOCATION_HAND, 0, 1, None);
        e.set_label(duel, 0);
        return Ok(res);
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_CONFIRM);
    let g = duel.select_matching_card(tp, |c, duel| cfilter(c, duel, e, tp), tp, LOCATION_HAND, 0, 1, 1, None)?;
    if let Some(rc) = g.get_first() {
        let labels = [rc.get_attribute(duel).into(), rc.get_code(duel).into()];
        e.set_labels(duel, &labels);
        duel.confirm_cards(1 - tp, rc)?;
    }
    duel.shuffle_hand(tp)?;
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, tp, LOCATION_DECK as i32);
    Ok(false)
}

fn activate(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    if duel.get_location_count(tp, LOCATION_MZONE) <= 0 {
        return Ok(());
    }
    let &[attr, code] = e.get_labels(duel) else {
        unported!(duel, true);
        return Ok(());
    };
    let (attr, code) = (attr as u32, code as u32);
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
    let g = duel.select_matching_card(tp, |c, duel| spfilter(c, duel, e, tp, attr, code), tp, LOCATION_DECK, 0, 1, 1, None)?;
    if let Some(tc) = g.get_first()
        && duel.special_summon_step(tc, 0, tp, tp, false, false, POS_FACEUP)?
    {
        aux::delayed_operation_full::<ReturnToHand>(duel, tc.into(), PHASE_END, 0, ID, e, tp, aux::stringid(ID, 1), 1);
    }
    duel.special_summon_complete()?;
    Ok(())
}
