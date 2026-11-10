// Tenpai Dragon Genroku
use crate::prelude::*;
use crate::testing::{Check, DECK, HAND, MZONE, OPPONENT};

const ID: u32 = 23657016;

pub const LISTED_NAMES: &[u32] = &[ID];

// On the field and in the Deck, with Shiina and a WIND monster on the field and Raigeki in the opponent's hand to return it to
// the hand without drawing it.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (ID, DECK),
    (91810826, DECK),
    (12197223, HAND),
    (3134241, MZONE),
    (12580477, HAND | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON + CATEGORY_LVCHANGE);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e1.set_property(duel, EFFECT_FLAG_DELAY);
    e1.set_code(duel, EVENT_TO_HAND);
    e1.set_count_limit(duel, 1, ID);
    e1.set_condition(duel, |duel, a| !a.e.get_handler(duel).is_reason(duel, REASON_DRAW));
    e1.set_target(duel, selfsptg);
    e1.set_operation(duel, selfspop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e2.set_type(duel, EFFECT_TYPE_QUICK_O);
    e2.set_code(duel, EVENT_FREE_CHAIN);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_hint_timing(
        duel,
        0,
        TIMING_MAIN_END | TIMING_BATTLE_START | TIMING_BATTLE_END | TIMINGS_CHECK_MONSTER_E,
    );
    e2.set_cost(duel, Cost::self_tribute);
    e2.set_target(duel, dsptg);
    e2.set_operation(duel, dspop);
    c.register_effect(duel, e2);
}

fn assume_tuner(duel: &Duel, c: CardId) {
    c.assume_property(duel, ASSUME_TYPE, u64::from(c.get_type(duel) | TYPE_TUNER));
}

fn selfsptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    assume_tuner(duel, c);
    if chk == 0 {
        return Ok(duel.get_location_count(tp, LOCATION_MZONE) > 0 && c.is_can_be_special_summoned(duel, e, 0, tp, false, false));
    }
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, Some(c.into()), 1, tp, 0);
    Ok(false)
}

fn selfspop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if !c.is_relate_to_effect(duel, e) {
        return Ok(());
    }
    assume_tuner(duel, c);
    if duel.special_summon(c, 0, tp, tp, false, false, POS_FACEUP)? > 0 {
        let e1 = duel.create_effect(c);
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE);
        e1.set_code(duel, EFFECT_ADD_TYPE);
        e1.set_value(duel, TYPE_TUNER as i32);
        e1.set_reset(duel, RESET_EVENT | RESETS_STANDARD);
        c.register_effect(duel, e1);
        if !duel.select_yes_no(tp, aux::stringid(ID, 2))? {
            return Ok(());
        }
        duel.break_effect()?;
        let e2 = e1.clone_effect(duel);
        e2.set_property(duel, 0);
        e2.set_code(duel, EFFECT_UPDATE_LEVEL);
        e2.set_value(duel, 1);
        e2.set_reset(duel, RESET_EVENT | RESETS_STANDARD_DISABLE);
        c.register_effect(duel, e2);
    }
    Ok(())
}

fn dspfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_set_card(duel, &[SET_TENPAI_DRAGON]) && !c.is_code(duel, ID) && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
}

fn dsptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if chk == 0 {
        let c = e.get_handler(duel);
        return Ok(duel.get_mzone_count(tp, Some(c.into())) > 0
            && duel.is_existing_matching_card(|c, duel| dspfilter(c, duel, e, tp), tp, LOCATION_DECK, 0, 1, None));
    }
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, tp, LOCATION_DECK as i32);
    Ok(false)
}

fn dspop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    if duel.get_location_count(tp, LOCATION_MZONE) > 0 {
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
        let g = duel.select_matching_card(tp, |c, duel| dspfilter(c, duel, e, tp), tp, LOCATION_DECK, 0, 1, 1, None)?;
        if !g.is_empty() {
            duel.special_summon(g, 0, tp, tp, false, false, POS_FACEUP)?;
        }
    }
    let e1 = duel.create_effect(e.get_handler(duel));
    e1.set_description(duel, aux::stringid(ID, 3));
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_property(duel, EFFECT_FLAG_PLAYER_TARGET + EFFECT_FLAG_CLIENT_HINT);
    e1.set_code(duel, EFFECT_CANNOT_SPECIAL_SUMMON);
    e1.set_target_range(duel, 1, 0);
    e1.set_target_card(duel, |duel, _, c| !c.is_race(duel, RACE_DRAGON));
    e1.set_reset(duel, RESET_PHASE | PHASE_END);
    duel.register_effect(e1, tp);
    Ok(())
}
