// Urgent Schedule
use crate::prelude::*;
use crate::testing::{Check, DECK, HAND, MZONE, OPPONENT, SZONE};

const ID: u32 = 25274141;

// In hand and Set, with an EARTH Machine of Level 4 and one of Level 5 in the Deck, more monsters on the opponent's side,
// Jizukiru (Level 10 Machine) in the Deck to search and the opponent's Mystical Space Typhoon to destroy the Set copy.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, SZONE),
    (2311603, DECK),
    (2311603, DECK),
    (29172562, DECK),
    (29172562, DECK),
    (63941210, DECK),
    (11091375, MZONE | OPPONENT),
    (11091375, MZONE | OPPONENT),
    (5318639, HAND | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_hint_timing(duel, 0, TIMINGS_CHECK_MONSTER_E);
    e1.set_count_limit(duel, 1, ID);
    e1.set_condition(duel, spcon);
    e1.set_cost(duel, spcost);
    e1.set_target(duel, sptg);
    e1.set_operation(duel, spop);
    c.register_effect(duel, e1);
    duel.add_custom_activity_counter(ID, ACTIVITY_ATTACK, CounterFilter::Card(|duel, c| c.is_race(duel, RACE_MACHINE)));
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_TOHAND + CATEGORY_SEARCH);
    e2.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e2.set_property(duel, EFFECT_FLAG_DAMAGE_STEP + EFFECT_FLAG_DELAY);
    e2.set_code(duel, EVENT_TO_GRAVE);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_condition(duel, |duel, a| {
        let c = a.e.get_handler(duel);
        c.is_previous_location(duel, LOCATION_ONFIELD) && c.is_previous_position(duel, POS_FACEDOWN)
    });
    e2.set_target(duel, thtg);
    e2.set_operation(duel, thop);
    c.register_effect(duel, e2);
}

fn spcon(duel: &Duel, a: &Args) -> bool {
    duel.get_field_group_count(a.tp, 0, LOCATION_MZONE) > duel.get_field_group_count(a.tp, LOCATION_MZONE, 0)
}

fn spcost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let (c, tp) = (a.e.get_handler(duel), a.tp);
    if chk == 0 {
        return Ok(duel.get_custom_activity_count(ID, tp, ACTIVITY_ATTACK) == 0);
    }
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_code(duel, EFFECT_CANNOT_ATTACK_ANNOUNCE);
    e1.set_property(duel, EFFECT_FLAG_IGNORE_IMMUNE + EFFECT_FLAG_OATH);
    e1.set_target_range(duel, LOCATION_MZONE, 0);
    e1.set_target_card(duel, |duel, _, c| !c.is_race(duel, RACE_MACHINE));
    e1.set_reset(duel, RESET_PHASE | PHASE_END);
    duel.register_effect(e1, tp);
    aux::register_client_hint(duel, c, 0, tp, 1, 0, Some(aux::stringid(ID, 2)));
    Ok(false)
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_race(duel, RACE_MACHINE)
        && c.is_attribute(duel, ATTRIBUTE_EARTH)
        && c.is_can_be_special_summoned_full(duel, e, 0, tp, false, false, POS_FACEUP_DEFENSE, tp, 0xff)
}

fn rescon(e: EffectId, tp: u8) -> impl Fn(&Duel, CardSet, CardSet, Option<CardId>) -> (bool, bool) {
    move |duel, sg, mg, c| {
        let count = |filter: &dyn Fn(CardId, &Duel) -> bool| sg.filter_count(duel, filter, None);
        let ok = aux::chkf_mmz(tp, 2)(duel, sg, mg, c).0
            && count(&|c, duel| spfilter(c, duel, e, tp)) == 2
            && count(&|c, duel| c.is_level_above(duel, 5)) == 1
            && count(&|c, duel| c.is_level_below(duel, 4)) == 1;
        (ok, false)
    }
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if chk == 0 {
        let g = duel.get_matching_group(|c, duel| spfilter(c, duel, e, tp), tp, LOCATION_DECK, 0, None);
        return Ok(aux::select_unselect_group_check(duel, g, 2, 2, &rescon(e, tp))
            && duel.get_location_count(tp, LOCATION_MZONE) > 1
            && duel.is_player_affected_by_effect(tp, CARD_BLUEEYES_SPIRIT).is_none());
    }
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 2, tp, LOCATION_DECK as i32);
    Ok(false)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if duel.is_player_affected_by_effect(tp, CARD_BLUEEYES_SPIRIT).is_some() || duel.get_location_count(tp, LOCATION_MZONE) < 2 {
        return Ok(());
    }
    let g = duel.get_matching_group(|c, duel| spfilter(c, duel, e, tp), tp, LOCATION_DECK, 0, None);
    let sg = aux::select_unselect_group(duel, g, 2, 2, &rescon(e, tp), tp, HINTMSG_SPSUMMON)?;
    if sg.is_empty() {
        return Ok(());
    }
    for tc in sg {
        if duel.special_summon_step(tc, 0, tp, tp, false, false, POS_FACEUP_DEFENSE)? {
            for code in [EFFECT_DISABLE, EFFECT_DISABLE_EFFECT] {
                let e1 = duel.create_effect(c);
                e1.set_type(duel, EFFECT_TYPE_SINGLE);
                e1.set_code(duel, code);
                e1.set_reset(duel, RESET_EVENT | RESETS_STANDARD);
                tc.register_effect(duel, e1);
            }
        }
    }
    duel.special_summon_complete()?;
    Ok(())
}

fn thfilter(c: CardId, duel: &Duel) -> bool {
    c.get_level(duel) == 10 && c.is_race(duel, RACE_MACHINE) && c.is_able_to_hand(duel)
}

fn thtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    stock::exists_target(duel, a, chk, thfilter, LOCATION_DECK, CATEGORY_TOHAND)
}

fn thop(duel: &mut Duel, a: &Args) -> Script<()> {
    stock::search(duel, a, thfilter, LOCATION_DECK)
}
