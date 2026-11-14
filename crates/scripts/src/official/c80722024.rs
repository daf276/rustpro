// Mementotlan Bone Party
use crate::prelude::*;
use crate::testing::{Check, DECK, GRAVE, HAND, MZONE};

const ID: u32 = 80722024;

pub const LISTED_SERIES: &[u16] = &[SET_MEMENTO];
// In the hand with Mementotlan Tatsunootoshigo on the field to destroy and Mementotlan Angwitch in the Deck to search, and in the
// Graveyard to give it piercing damage.
pub const SUPPORT: &[(u32, u16)] = &[(ID, HAND), (ID, GRAVE), (81677154, MZONE), (54550967, DECK), (54550967, DECK)];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON + CATEGORY_SEARCH + CATEGORY_TOHAND + CATEGORY_DESTROY);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_hint_timing(duel, 0, TIMINGS_CHECK_MONSTER_E);
    e1.set_count_limit(duel, 1, ID);
    e1.set_target(duel, target);
    e1.set_operation(duel, activate);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_type(duel, EFFECT_TYPE_IGNITION);
    e2.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e2.set_range(duel, LOCATION_GRAVE);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_cost(duel, Cost::self_banish);
    e2.set_target(duel, pdtg);
    e2.set_operation(duel, pdop);
    c.register_effect(duel, e2);
}

fn thspfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8, code: u32, ft: i32) -> bool {
    c.is_set_card(duel, &[SET_MEMENTO])
        && c.is_monster(duel)
        && !c.is_code(duel, code)
        && (c.is_able_to_hand(duel)
            || (ft > 0 && c.is_can_be_special_summoned_full(duel, e, 0, tp, false, false, POS_FACEUP_DEFENSE, tp, 0xff)))
}

fn desfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_set_card(duel, &[SET_MEMENTO]) && c.is_monster(duel) && (c.is_location(duel, LOCATION_HAND) || c.is_faceup(duel)) && {
        let (code, ft) = (c.get_code(duel), duel.get_mzone_count(tp, Some(c.into())));
        duel.is_existing_matching_card(|sc, duel| thspfilter(sc, duel, e, tp, code, ft), tp, LOCATION_DECK, 0, 1, None)
    }
}

fn target(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(|c, duel| desfilter(c, duel, e, tp), tp, LOCATION_HAND | LOCATION_MZONE, 0, 1, None));
    }
    duel.set_operation_info(0, CATEGORY_DESTROY, None, 1, tp, (LOCATION_HAND | LOCATION_MZONE) as i32);
    duel.set_possible_operation_info(0, CATEGORY_TOHAND, None, 1, tp, LOCATION_DECK as i32);
    duel.set_possible_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, tp, LOCATION_DECK as i32);
    Ok(false)
}

fn activate(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_DESTROY);
    let g = duel.select_matching_card(
        tp,
        |c, duel| desfilter(c, duel, e, tp),
        tp,
        LOCATION_HAND | LOCATION_MZONE,
        0,
        1,
        1,
        None,
    )?;
    let Some(tc) = g.get_first() else { return Ok(()) };
    if duel.destroy(tc, REASON_EFFECT)? == 0 {
        return Ok(());
    }
    let ft = duel.get_location_count(tp, LOCATION_MZONE);
    duel.hint(HINT_SELECTMSG, tp, aux::stringid(ID, 2));
    let code = tc.get_code(duel);
    let g = duel.select_matching_card(tp, |c, duel| thspfilter(c, duel, e, tp, code, ft), tp, LOCATION_DECK, 0, 1, 1, None)?;
    let Some(sc) = g.get_first() else { return Ok(()) };
    aux::to_hand_or_else(
        duel,
        sc,
        tp,
        |_, duel| ft > 0 && sc.is_can_be_special_summoned_full(duel, e, 0, tp, false, false, POS_FACEUP_DEFENSE, tp, 0xff),
        |duel| duel.special_summon(sc, 0, tp, tp, false, false, POS_FACEUP_DEFENSE).map(drop),
        aux::stringid(ID, 3),
    )
}

fn cfilter(c: CardId, duel: &Duel) -> bool {
    c.is_faceup(duel) && c.is_set_card(duel, &[SET_MEMENTO]) && c.is_has_effect(duel, EFFECT_PIERCE).is_none()
}

fn pdtg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_MZONE) && chkc.is_controler(duel, tp) && cfilter(chkc, duel));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(cfilter, tp, LOCATION_MZONE, 0, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TARGET);
    duel.select_target(tp, cfilter, tp, LOCATION_MZONE, 0, 1, 1, None)?;
    Ok(false)
}

fn pdop(duel: &mut Duel, a: &Args) -> Script<()> {
    if let Some(tc) = duel.get_first_target()
        && tc.is_relate_to_effect(duel, a.e)
        && tc.is_faceup(duel)
    {
        let e1 = duel.create_effect(a.e.get_handler(duel));
        e1.set_description(duel, 3208);
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_property(duel, EFFECT_FLAG_CLIENT_HINT);
        e1.set_code(duel, EFFECT_PIERCE);
        e1.set_reset(duel, RESETS_STANDARD_PHASE_END);
        tc.register_effect(duel, e1);
    }
    Ok(())
}
