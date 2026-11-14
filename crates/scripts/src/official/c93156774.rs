// Vanquish Soul Hollie Sue
use crate::prelude::*;
use crate::testing::{Check, DECK, HAND, MZONE, OPPONENT};

const ID: u32 = 93156774;

pub const LISTED_SERIES: &[u16] = &[SET_VANQUISH_SOUL];
// Hollie Sue with Jiaolong (FIRE), Dr. Mad Love (DARK) and another Hollie Sue (EARTH) in the hand to reveal, and Jiaolong in the Deck.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, MZONE),
    (29280200, HAND),
    (29280200, HAND),
    (9091064, HAND),
    (9091064, DECK),
    (3573512, MZONE | OPPONENT),
    (4035199, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e1.set_type(duel, EFFECT_TYPE_QUICK_O);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_range(duel, LOCATION_HAND);
    e1.set_hint_timing(duel, 0, TIMING_MAIN_END | TIMINGS_CHECK_MONSTER);
    e1.set_count_limit(duel, 1, ID);
    e1.set_condition(duel, |duel, _| duel.is_main_phase());
    e1.set_cost(duel, |duel, a, chk| {
        Cost::and(duel, a, chk, &[Cost::hard_once_per_chain::<ID>, revealcost])
    });
    e1.set_target(duel, stock::special_summon_self_target);
    e1.set_operation(duel, stock::special_summon_self::<POS_FACEUP>);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_type(duel, EFFECT_TYPE_QUICK_O);
    e2.set_code(duel, EVENT_FREE_CHAIN);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_cost(duel, Cost::hard_once_per_chain::<ID>);
    e2.set_target(duel, vstg);
    e2.set_operation(duel, vsop);
    e2.set_hint_timing(duel, 0, TIMING_STANDBY_PHASE | TIMING_MAIN_END | TIMINGS_CHECK_MONSTER_E);
    c.register_effect(duel, e2);
}

fn revealfilter(c: CardId, duel: &Duel, _: EffectId, _: u8) -> bool {
    c.is_set_card(duel, &[SET_VANQUISH_SOUL]) && c.is_monster(duel)
}

fn revealcost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    Cost::reveal(duel, a, chk, revealfilter, true, (1, 1), None, LOCATION_HAND)
}

fn vscostfilter(c: CardId, duel: &Duel, att: u32) -> bool {
    c.is_attribute(duel, att) && !c.is_public(duel)
}

fn vsrescon(duel: &Duel, sg: CardSet) -> (bool, bool) {
    (sg.get_bin_class_count(duel, |c, duel| c.get_attribute(duel).into()) == 2, false)
}

fn deckspfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_set_card(duel, &[SET_VANQUISH_SOUL])
        && !c.is_race(duel, RACE_PSYCHIC)
        && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
}

fn vstg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let cg1 = duel.get_matching_group(
        |c, duel| vscostfilter(c, duel, ATTRIBUTE_EARTH | ATTRIBUTE_DARK),
        tp,
        LOCATION_HAND,
        0,
        None,
    );
    let b1 = aux::select_unselect_group_check(duel, cg1, 2, 2, &|duel, sg, _, _| vsrescon(duel, sg))
        && duel.is_existing_matching_card(
            aux::faceup_filter(CardId::is_controler_can_be_changed),
            tp,
            0,
            LOCATION_MZONE,
            1,
            None,
        );
    let cg2 = duel.get_matching_group(
        |c, duel| vscostfilter(c, duel, ATTRIBUTE_FIRE | ATTRIBUTE_DARK),
        tp,
        LOCATION_HAND,
        0,
        None,
    );
    let b2 = aux::select_unselect_group_check(duel, cg2, 2, 2, &|duel, sg, _, _| vsrescon(duel, sg))
        && duel.get_location_count(tp, LOCATION_MZONE) > 0
        && duel.is_existing_matching_card(|c, duel| deckspfilter(c, duel, e, tp), tp, LOCATION_DECK, 0, 1, None);
    if chk == 0 {
        return Ok(b1 || b2);
    }
    let op = duel.select_effect(tp, &[(b1, aux::stringid(ID, 2)), (b2, aux::stringid(ID, 3))])?;
    e.set_label(duel, op.map_or(0, |op| op as i64));
    if let Some(op) = op {
        let g = aux::select_unselect_group(
            duel,
            if op == 1 { cg1 } else { cg2 },
            2,
            2,
            &|duel, sg, _, _| vsrescon(duel, sg),
            tp,
            HINTMSG_CONFIRM,
        )?;
        duel.confirm_cards(1 - tp, g)?;
        duel.shuffle_hand(tp)?;
        if op == 1 {
            e.set_category(duel, CATEGORY_CONTROL);
            duel.set_operation_info(0, CATEGORY_CONTROL, None, 1, 1 - tp, LOCATION_MZONE as i32);
        } else {
            e.set_category(duel, CATEGORY_SPECIAL_SUMMON);
            duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, tp, LOCATION_DECK as i32);
        }
    }
    Ok(false)
}

fn vsop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let op = e.get_label(duel);
    if op == 1 {
        let g = duel.get_matching_group(aux::faceup_filter(CardId::is_controler_can_be_changed), tp, 0, LOCATION_MZONE, None);
        let Some((ming, _)) = g.get_min_group(duel, |c, duel| c.get_attack(duel).into()) else {
            return Ok(());
        };
        let mut sc = ming.get_first();
        if ming.len() > 1 {
            duel.hint(HINT_SELECTMSG, tp, HINTMSG_CONTROL);
            sc = ming.select(duel, tp, 1, 1, None)?.get_first();
        }
        if let Some(sc) = sc {
            duel.hint_selection(sc);
            duel.get_control_full(sc, tp, PHASE_END as u16, 1, 0xff, duel.get_reason_player())?;
        }
    } else if op == 2 {
        if duel.get_location_count(tp, LOCATION_MZONE) == 0 {
            return Ok(());
        }
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
        let g = duel.select_matching_card(tp, |c, duel| deckspfilter(c, duel, e, tp), tp, LOCATION_DECK, 0, 1, 1, None)?;
        if !g.is_empty() {
            duel.special_summon(g, 0, tp, tp, false, false, POS_FACEUP)?;
        }
    }
    Ok(())
}
