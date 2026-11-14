// DoomZ Raiders
use crate::prelude::*;
use crate::testing::{Check, DECK, HAND, MZONE, OPPONENT};

const ID: u32 = 32442017;

pub const LISTED_SERIES: &[u16] = &[SET_DOOMZ];
// In the hand, with DoomZ monsters on the field and in the hand to destroy and in the Deck to add or Special Summon, and the
// opponent's monsters to destroy once it is destroyed by Dark Hole.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (54265980, MZONE),
    (28877382, HAND),
    (28877382, DECK),
    (28877382, DECK),
    (53129443, HAND),
    (7892180, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e0 = duel.create_effect(c);
    e0.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e0.set_code(duel, EVENT_FREE_CHAIN);
    c.register_effect(duel, e0);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_DESTROY + CATEGORY_TOHAND + CATEGORY_SEARCH + CATEGORY_SPECIAL_SUMMON);
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_range(duel, LOCATION_SZONE);
    e1.set_count_limit(duel, 1, ID);
    e1.set_target(duel, desthsptg);
    e1.set_operation(duel, desthspop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_DESTROY);
    e2.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e2.set_property(duel, EFFECT_FLAG_DELAY + EFFECT_FLAG_CARD_TARGET);
    e2.set_code(duel, EVENT_DESTROYED);
    e2.set_count_limit(duel, 1, ID);
    e2.set_condition(duel, |_, a| a.r & REASON_EFFECT > 0);
    e2.set_target(duel, destg);
    e2.set_operation(duel, stock::destroy_first_target);
    c.register_effect(duel, e2);
}

fn desfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    if !(c.is_set_card(duel, &[SET_DOOMZ]) && (c.is_faceup(duel) || c.is_location(duel, LOCATION_HAND))) {
        return false;
    }
    duel.is_existing_matching_card(|sc, duel| thspfilter(sc, duel, e, tp, c), tp, LOCATION_DECK, 0, 1, None)
}

fn thspfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8, exc: CardId) -> bool {
    c.is_set_card(duel, &[SET_DOOMZ])
        && c.is_monster(duel)
        && (c.is_able_to_hand(duel)
            || (c.is_can_be_special_summoned(duel, e, 0, tp, false, false) && duel.get_mzone_count(tp, Some(exc.into())) > 0))
}

fn desthsptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if chk == 0 {
        let c = e.get_handler(duel);
        return Ok(duel.is_existing_matching_card(
            |c, duel| desfilter(c, duel, e, tp),
            tp,
            LOCATION_HAND | LOCATION_ONFIELD,
            0,
            1,
            Some(c.into()),
        ));
    }
    duel.set_operation_info(0, CATEGORY_DESTROY, None, 1, tp, (LOCATION_HAND | LOCATION_ONFIELD) as i32);
    duel.set_possible_operation_info(0, CATEGORY_TOHAND, None, 1, tp, LOCATION_DECK as i32);
    duel.set_possible_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, tp, LOCATION_DECK as i32);
    Ok(false)
}

fn desthspop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_DESTROY);
    let g = duel.select_matching_card(
        tp,
        |c, duel| desfilter(c, duel, e, tp),
        tp,
        LOCATION_HAND | LOCATION_ONFIELD,
        0,
        1,
        1,
        Some(c.into()),
    )?;
    if !g.is_empty() && duel.destroy(g, REASON_EFFECT)? > 0 {
        duel.hint(HINT_SELECTMSG, tp, aux::stringid(ID, 2));
        let filter = |sc, duel: &Duel| thspfilter(sc, duel, e, tp, c);
        if let Some(sc) = duel.select_matching_card(tp, filter, tp, LOCATION_DECK, 0, 1, 1, None)?.get_first() {
            duel.break_effect()?;
            let check = |_, duel: &Duel| {
                duel.get_location_count(tp, LOCATION_MZONE) > 0 && sc.is_can_be_special_summoned(duel, e, 0, tp, false, false)
            };
            let spsummon = |duel: &mut Duel| duel.special_summon(sc, 0, tp, tp, false, false, POS_FACEUP).map(|_| ());
            aux::to_hand_or_else(duel, sc, tp, check, spsummon, aux::stringid(ID, 3))?;
        }
    }
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 4));
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_property(duel, EFFECT_FLAG_PLAYER_TARGET + EFFECT_FLAG_CLIENT_HINT);
    e1.set_code(duel, EFFECT_CANNOT_SPECIAL_SUMMON);
    e1.set_target_range(duel, 1, 0);
    e1.set_target_card(duel, |duel, _, c| c.is_location(duel, LOCATION_EXTRA) && !c.is_type(duel, TYPE_XYZ));
    e1.set_reset(duel, RESET_PHASE | PHASE_END);
    duel.register_effect(e1, tp);
    aux::add_temp_lizard_check_full(duel, c, tp, |duel, _, c| !c.is_original_type(duel, TYPE_XYZ));
    Ok(())
}

fn destg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_MZONE) && chkc.is_faceup(duel));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(CardId::is_faceup, tp, LOCATION_MZONE, LOCATION_MZONE, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_DESTROY);
    let g = duel.select_target(tp, CardId::is_faceup, tp, LOCATION_MZONE, LOCATION_MZONE, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_DESTROY, g, 1, tp, 0);
    Ok(false)
}
