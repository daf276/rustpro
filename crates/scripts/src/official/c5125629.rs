// Malefic Paradigm Shift
use crate::prelude::*;
use crate::testing::{Check, DECK, EXTRA, HAND};

const ID: u32 = 5125629;
const MALEFIC_WORLD: u32 = 27564031;

pub const LISTED_NAMES: &[u32] = &[MALEFIC_WORLD];

// In the hand, with Malefic World in the Deck to find and Malefic monsters in the Deck and Extra Deck to Special Summon.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, HAND),
    (MALEFIC_WORLD, DECK),
    (MALEFIC_WORLD, DECK),
    (48829461, DECK),
    (74509280, DECK),
    (8310162, EXTRA),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 1))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e0 = duel.create_effect(c);
    e0.set_description(duel, aux::stringid(ID, 0));
    e0.set_type(duel, EFFECT_TYPE_SINGLE);
    e0.set_code(duel, EFFECT_TRAP_ACT_IN_HAND);
    e0.set_value(duel, Value::Activation(|duel, e, _| e.set_label(duel, 1)));
    c.register_effect(duel, e0);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 1));
    e1.set_category(
        duel,
        CATEGORY_TOHAND + CATEGORY_SEARCH + CATEGORY_SPECIAL_SUMMON + CATEGORY_ATKCHANGE,
    );
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_count_limit_full(duel, 1, ID, EFFECT_COUNT_CODE_OATH);
    e1.set_cost(duel, cost);
    e1.set_target(duel, target);
    e1.set_operation(duel, activate);
    e1.set_label_object(duel, Some(e0));
    e1.set_hint_timing(duel, 0, TIMING_STANDBY_PHASE | TIMING_MAIN_END | TIMINGS_CHECK_MONSTER_E);
    c.register_effect(duel, e1);
}

fn cost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let Some(label_obj) = a.e.get_label_object(duel) else {
        return Ok(chk == 0);
    };
    if chk == 0 {
        label_obj.set_label(duel, 0);
        return Ok(true);
    }
    if label_obj.get_label(duel) == 1 {
        label_obj.set_label(duel, 0);
        let cost = duel.get_lp(a.tp) / 2;
        duel.pay_lp_cost(a.tp, cost as u32)?;
    }
    Ok(false)
}

fn maleficworldfilter(c: CardId, duel: &Duel) -> bool {
    c.is_code(duel, MALEFIC_WORLD) && (c.is_able_to_hand(duel) || !c.is_forbidden(duel))
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    if !(c.is_set_card(duel, &[SET_MALEFIC])
        && c.is_monster(duel)
        && c.is_can_be_special_summoned_full(duel, e, 0, tp, true, false, POS_FACEUP_DEFENSE, tp, 0xff))
    {
        return false;
    }
    if c.is_location(duel, LOCATION_DECK) {
        duel.get_location_count(tp, LOCATION_MZONE) > 0
    } else if c.is_location(duel, LOCATION_EXTRA) {
        duel.get_location_count_from_ex(tp, tp, None, c) > 0
    } else {
        false
    }
}

fn target(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if chk == 0 {
        return Ok(
            duel.is_existing_matching_card(maleficworldfilter, tp, LOCATION_DECK | LOCATION_GRAVE, 0, 1, None)
                && duel.is_existing_matching_card(|c, duel| spfilter(c, duel, e, tp), tp, LOCATION_DECK | LOCATION_EXTRA, 0, 1, None),
        );
    }
    duel.set_operation_info(0, CATEGORY_TOHAND, None, 1, tp, (LOCATION_DECK | LOCATION_GRAVE) as i32);
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, tp, (LOCATION_DECK | LOCATION_EXTRA) as i32);
    let g = duel.get_field_group(tp, 0, LOCATION_MZONE);
    duel.set_possible_operation_info(0, CATEGORY_ATKCHANGE, Some(g), 1, tp, -2500);
    Ok(false)
}

fn activate(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    duel.hint(HINT_SELECTMSG, tp, aux::stringid(ID, 2));
    let filter = aux::necro_valley_filter(maleficworldfilter);
    let Some(sc) = duel
        .select_matching_card(tp, filter, tp, LOCATION_DECK | LOCATION_GRAVE, 0, 1, 1, None)?
        .get_first()
    else {
        return Ok(());
    };
    let tohand_or_place_chk = aux::to_hand_or_else_result(
        duel,
        sc,
        tp,
        |_, duel| !sc.is_forbidden(duel),
        |duel| duel.place_field_spell(sc, tp),
        aux::stringid(ID, 3),
    )?;
    if !tohand_or_place_chk {
        return Ok(());
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
    let sg = duel.select_matching_card(
        tp,
        |c, duel| spfilter(c, duel, e, tp),
        tp,
        LOCATION_DECK | LOCATION_EXTRA,
        0,
        1,
        1,
        None,
    )?;
    if sg.is_empty() {
        return Ok(());
    }
    duel.break_effect()?;
    if duel.special_summon(sg, 0, tp, tp, true, false, POS_FACEUP_DEFENSE)? > 0
        && duel.is_existing_matching_card(
            aux::faceup_filter(|c, duel| c.is_attack_above(duel, 2500)),
            tp,
            0,
            LOCATION_MZONE,
            1,
            None,
        )
    {
        let g = duel.get_matching_group(CardId::is_faceup, tp, 0, LOCATION_MZONE, None);
        let c = e.get_handler(duel);
        duel.break_effect()?;
        for tc in g {
            tc.update_attack(duel, -2500, Some(RESETS_STANDARD_PHASE_END), Some(c), None);
        }
    }
    Ok(())
}
