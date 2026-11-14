// Incredible Ecclesia, the Virtuous
use crate::prelude::*;
use crate::testing::{Check, DECK, GRAVE, HAND, MZONE, OPPONENT};

const ID: u32 = 55273560;

pub const LISTED_SERIES: &[u16] = &[SET_SWORDSOUL];
pub const LISTED_NAMES: &[u32] = &[CARD_ALBAZ];

// In the hand to Special Summon with fewer monsters than the opponent, on the field to tribute for Fallen of Albaz in the Deck, and
// in the Graveyard with Albion destroyed by the opponent's Dark Hole.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, MZONE),
    (ID, GRAVE),
    (CARD_ALBAZ, DECK),
    (CARD_ALBAZ, DECK),
    (87746184, MZONE),
    (26202165, MZONE | OPPONENT),
    (26202165, MZONE | OPPONENT),
    (53129443, HAND | OPPONENT),
];
// Both effects share a description, so one check covers them.
pub const CHECKS: &[Check] = &[
    Check::SpecialSummoned,
    Check::SpecialSummonedCode(CARD_ALBAZ),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_code(duel, EFFECT_SPSUMMON_PROC);
    e1.set_property(duel, EFFECT_FLAG_UNCOPYABLE);
    e1.set_range(duel, LOCATION_HAND);
    e1.set_count_limit_full(duel, 1, ID, EFFECT_COUNT_CODE_OATH);
    e1.set_condition(duel, spcon);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e2.set_type(duel, EFFECT_TYPE_QUICK_O);
    e2.set_code(duel, EVENT_FREE_CHAIN);
    e2.set_hint_timing(duel, 0, TIMINGS_CHECK_MONSTER | TIMING_MAIN_END);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_condition(duel, |duel, _| duel.is_main_phase());
    e2.set_cost(duel, Cost::self_tribute);
    e2.set_target(duel, sstg);
    e2.set_operation(duel, ssop);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 1));
    e3.set_category(duel, CATEGORY_TOHAND);
    e3.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e3.set_code(duel, EVENT_PHASE + PHASE_END);
    e3.set_range(duel, LOCATION_GRAVE);
    e3.set_count_limit(duel, 1, (ID, 2));
    e3.set_condition(duel, |duel, a| duel.get_flag_effect(a.tp, ID) > 0);
    e3.set_target(duel, thtg);
    e3.set_operation(duel, stock::self_to_hand);
    c.register_effect(duel, e3);
    aux::global_check(duel, ID, |duel| {
        let ge1 = duel.create_effect(c);
        ge1.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_CONTINUOUS);
        ge1.set_code(duel, EVENT_TO_GRAVE);
        ge1.set_operation(duel, checkop);
        duel.register_effect(ge1, 0);
    });
}

fn checkop(duel: &mut Duel, a: &Args) -> Script<()> {
    for tc in a.eg.unwrap_or_default() {
        if tc.is_type(duel, TYPE_FUSION) {
            duel.register_flag_effect(tc.get_controler(duel), ID, RESET_PHASE | PHASE_END, 0, 1);
        }
    }
    Ok(())
}

fn spcon(duel: &Duel, a: &Args) -> bool {
    let Some(c) = a.c else { return true };
    let tp = c.get_controler(duel);
    duel.get_location_count(tp, LOCATION_MZONE) > 0
        && duel.get_field_group_count(tp, LOCATION_MZONE, 0) < duel.get_field_group_count(tp, 0, LOCATION_MZONE)
}

fn ssfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    (c.is_code(duel, CARD_ALBAZ) || c.is_set_card(duel, &[SET_SWORDSOUL])) && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
}

fn sstg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if chk == 0 {
        return Ok(duel.get_mzone_count(tp, Some(e.get_handler(duel).into())) > 0
            && duel.is_existing_matching_card(|c, duel| ssfilter(c, duel, e, tp), tp, LOCATION_HAND | LOCATION_DECK, 0, 1, None));
    }
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, tp, (LOCATION_HAND | LOCATION_DECK) as i32);
    Ok(false)
}

fn ssop(duel: &mut Duel, a: &Args) -> Script<()> {
    stock::special_summon(
        duel,
        a,
        |c, duel| ssfilter(c, duel, a.e, a.tp),
        LOCATION_HAND | LOCATION_DECK,
        POS_FACEUP,
    )
}

fn thtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let c = a.e.get_handler(duel);
    if chk == 0 {
        return Ok(c.is_able_to_hand(duel));
    }
    duel.set_operation_info(0, CATEGORY_TOHAND, Some(c.into()), 1, 0, 0);
    Ok(false)
}
