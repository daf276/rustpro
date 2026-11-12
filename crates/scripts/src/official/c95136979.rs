// Unchained Ogre Shara
use crate::prelude::*;
use crate::testing::{Check, GRAVE, HAND, MZONE, OPPONENT};

const ID: u32 = 95136979;

pub const LISTED_NAMES: &[u32] = &[ID];

// In the hand, with a Fiend in the hand to summon, and in the Graveyard with a copy on the field and Dark Hole in the opponent's hand.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, HAND),
    (ID, GRAVE),
    (26236560, HAND),
    (26236560, MZONE),
    (53129443, HAND | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON + CATEGORY_DESTROY);
    e1.set_type(duel, EFFECT_TYPE_QUICK_O);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_range(duel, LOCATION_HAND);
    e1.set_count_limit(duel, 1, (ID, 0));
    e1.set_condition(duel, |duel, _| duel.is_main_phase());
    e1.set_cost(duel, Cost::self_discard);
    e1.set_target(duel, handsptg);
    e1.set_operation(duel, handspop);
    e1.set_hint_timing(duel, 0, TIMING_MAIN_END | TIMINGS_CHECK_MONSTER);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_SPECIAL_SUMMON + CATEGORY_TOHAND + CATEGORY_TODECK);
    e2.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e2.set_property_full(duel, EFFECT_FLAG_DELAY + EFFECT_FLAG_DAMAGE_STEP, EFFECT_FLAG2_CHECK_SIMULTANEOUS);
    e2.set_code(duel, EVENT_DESTROYED);
    e2.set_range(duel, LOCATION_GRAVE);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_condition(duel, selfspthcon);
    e2.set_target(duel, selfspthtg);
    e2.set_operation(duel, selfspthop);
    c.register_effect(duel, e2);
}

fn handspfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_race(duel, RACE_FIEND) && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
}

fn handsptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if chk == 0 {
        return Ok(duel.get_location_count(tp, LOCATION_MZONE) > 0
            && duel.is_existing_matching_card(|c, duel| handspfilter(c, duel, e, tp), tp, LOCATION_HAND, 0, 1, Some(c.into())));
    }
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, tp, LOCATION_HAND as i32);
    let g = duel.get_field_group(tp, LOCATION_ONFIELD, 0);
    if !g.is_empty() {
        duel.set_operation_info(0, CATEGORY_DESTROY, Some(g), 1, tp, 0);
    }
    Ok(false)
}

fn handspop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    if duel.get_location_count(tp, LOCATION_MZONE) <= 0 {
        return Ok(());
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
    let g = duel.select_matching_card(tp, |c, duel| handspfilter(c, duel, e, tp), tp, LOCATION_HAND, 0, 1, 1, None)?;
    if !g.is_empty() && duel.special_summon(g, 0, tp, tp, false, false, POS_FACEUP)? > 0 {
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_DESTROY);
        let dg = duel.select_matching_card(tp, aux::TRUE, tp, LOCATION_ONFIELD, 0, 1, 1, None)?;
        if !dg.is_empty() {
            duel.hint_selection(dg);
            duel.break_effect()?;
            duel.destroy(dg, REASON_EFFECT)?;
        }
    }
    Ok(())
}

fn selfspthconfilter(c: CardId, duel: &Duel) -> bool {
    c.is_previous_location(duel, LOCATION_ONFIELD) && c.is_reason(duel, REASON_BATTLE | REASON_EFFECT)
}

fn selfspthcon(duel: &Duel, a: &Args) -> bool {
    let eg = a.eg.unwrap_or_default();
    if eg.is_contains(a.e.get_handler(duel)) || !eg.is_exists(duel, selfspthconfilter, 1, None) {
        return false;
    }
    if a.r & REASON_BATTLE > 0 {
        return true;
    }
    if duel.is_chain_solving() {
        unported!(duel, a.re.is_none());
        let Some(re) = a.re else { return false };
        let rc = re.get_handler(duel);
        if rc.is_relate_to_effect(duel, re) && rc.is_faceup(duel) {
            !rc.is_code(duel, ID)
        } else {
            let code1: u32 = duel.get_chain_info(0, CHAININFO_TRIGGERING_CODE);
            let code2: u32 = duel.get_chain_info(0, CHAININFO_TRIGGERING_CODE2);
            code1 != ID && code2 != ID
        }
    } else if let Some(re) = a.re {
        !re.get_handler(duel).is_code(duel, ID)
    } else {
        !eg.get_first().is_some_and(|tc| tc.is_previous_code_on_field(duel, &[ID]))
    }
}

fn selfspthtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if chk == 0 {
        return Ok(c.is_able_to_hand(duel)
            || (duel.get_location_count(tp, LOCATION_MZONE) > 0 && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)));
    }
    duel.set_possible_operation_info(0, CATEGORY_SPECIAL_SUMMON, Some(c.into()), 1, tp, 0);
    duel.set_possible_operation_info(0, CATEGORY_TOHAND, Some(c.into()), 1, tp, 0);
    Ok(false)
}

fn selfspthop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if !c.is_relate_to_effect(duel, e) {
        return Ok(());
    }
    aux::to_hand_or_else(
        duel,
        c,
        tp,
        |_, duel| duel.get_location_count(tp, LOCATION_MZONE) > 0 && c.is_can_be_special_summoned(duel, e, 0, tp, false, false),
        |duel| {
            if duel.special_summon(c, 0, tp, tp, false, false, POS_FACEUP)? > 0 {
                let e1 = duel.create_effect(c);
                e1.set_description(duel, aux::stringid(ID, 2));
                e1.set_type(duel, EFFECT_TYPE_SINGLE);
                e1.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE + EFFECT_FLAG_CLIENT_HINT);
                e1.set_code(duel, EFFECT_LEAVE_FIELD_REDIRECT);
                e1.set_value(duel, LOCATION_DECKBOT as i32);
                e1.set_reset(duel, RESET_EVENT | RESETS_REDIRECT);
                c.register_effect_full(duel, e1, true);
            }
            Ok(())
        },
        aux::stringid(ID, 3),
    )
}
