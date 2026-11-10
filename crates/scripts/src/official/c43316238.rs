// Danger! Bigfoot!
use crate::prelude::*;
use crate::testing::{Check, DECK, HAND, MZONE, OPPONENT};

const ID: u32 = 43316238;

pub const LISTED_NAMES: &[u32] = &[ID];
// Two copies in the hand to discard and Special Summon, and Sangan on the opponent's field to destroy.
pub const SUPPORT: &[(u32, u16)] = &[(ID, HAND), (ID, HAND), (ID, DECK), (26202165, MZONE | OPPONENT)];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON + CATEGORY_HANDES + CATEGORY_DRAW);
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_range(duel, LOCATION_HAND);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_cost(duel, spcost::<ID>);
    e1.set_target(duel, sptg);
    e1.set_operation(duel, spop::<ID>);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_DESTROY);
    e2.set_property(duel, EFFECT_FLAG_CARD_TARGET + EFFECT_FLAG_DELAY);
    e2.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e2.set_code(duel, EVENT_TO_GRAVE);
    e2.set_count_limit(duel, 1, ID);
    e2.set_condition(duel, descon);
    e2.set_target(duel, destg);
    e2.set_operation(duel, desop);
    c.register_effect(duel, e2);
    let e3 = e2.clone_effect(duel);
    e3.set_code(duel, EVENT_REMOVE);
    c.register_effect(duel, e3);
}

// Shared with the other Danger! monsters that discard at random to Special Summon a copy of themselves.
pub fn spcost<const CODE: u32>(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let c = a.e.get_handler(duel);
    if chk == 0 {
        return Ok(!c.is_public(duel) && c.get_flag_effect(duel, CODE) == 0);
    }
    c.register_flag_effect(duel, CODE, RESET_CHAIN, 0, 1);
    Ok(false)
}

fn spfilter<const CODE: u32>(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_code(duel, CODE) && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
}

pub fn sptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(true);
    }
    duel.set_operation_info(0, CATEGORY_HANDES, None, 0, a.tp, 1);
    Ok(false)
}

pub fn spop<const CODE: u32>(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let g = duel.get_field_group(tp, LOCATION_HAND, 0);
    if g.is_empty() {
        return Ok(());
    }
    duel.shuffle_hand(tp)?;
    duel.hint(HINT_SELECTMSG, 1 - tp, HINTMSG_DISCARD);
    let tc = g.random_select(duel, 1 - tp, 1);
    duel.break_effect()?;
    duel.send_to_grave(tc, REASON_EFFECT | REASON_DISCARD)?;
    if !duel.is_player_can_special_summon_any(tp) || duel.get_location_count(tp, LOCATION_MZONE) == 0 {
        return Ok(());
    }
    if !tc.get_first().is_some_and(|c| c.is_code(duel, CODE)) {
        duel.break_effect()?;
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
        let sc = duel.select_matching_card(tp, |c, duel| spfilter::<CODE>(c, duel, e, tp), tp, LOCATION_HAND, 0, 1, 1, None)?;
        if !sc.is_empty() {
            duel.special_summon(sc, 0, tp, tp, false, false, POS_FACEUP)?;
            duel.draw(tp, 1, REASON_EFFECT)?;
        }
    }
    Ok(())
}

pub fn descon(duel: &Duel, a: &Args) -> bool {
    a.e.get_handler(duel).get_previous_location(duel) == LOCATION_HAND && a.r & REASON_DISCARD != 0
}

fn destg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_on_field(duel) && chkc.is_controler(duel, 1 - tp) && chkc.is_faceup(duel));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(CardId::is_faceup, tp, 0, LOCATION_ONFIELD, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_DESTROY);
    let g = duel.select_target(tp, CardId::is_faceup, tp, 0, LOCATION_ONFIELD, 1, 1, None)?;
    let count = g.map_or(0, |g| g.len() as u8);
    duel.set_operation_info(0, CATEGORY_DESTROY, g, count, 0, 0);
    Ok(false)
}

fn desop(duel: &mut Duel, a: &Args) -> Script<()> {
    if let Some(tc) = duel.get_first_target()
        && tc.is_relate_to_effect(duel, a.e)
    {
        duel.destroy(tc, REASON_EFFECT)?;
    }
    Ok(())
}
